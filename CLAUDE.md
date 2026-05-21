# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Rules

- 打印日志使用 `log` crate 的宏（`log::info!`、`log::error!`、`log::warn!`、`log::debug!`），不要用 `println!`
- 每次新增 `#[command]` 方法，必须同步在以下三个文件中添加权限：
  - `permissions/default.toml` — 新增 `[[permission]]` 块，`identifier` 命名为 `allow-<command-name>`（kebab-case），`commands.allow` 指向命令名（snake_case）
  - `capabilities/default.json` — 在 `permissions` 数组中添加 `"allow-<command-name>"`
  - `capabilities/douyin-remote.json` — 在 `permissions` 数组中添加 `"allow-<command-name>"`（如果该命令需要从远程 webview 调用）

## Build / Run

```bash
# Rust type-check (fast, no codegen)
cargo check

# Full build
cargo build

# Run the Tauri dev server (includes frontend hot-reload)
cd .. && npm run tauri dev

# The Rust workspace root is src-tauri/
cd src-tauri
```

No tests exist yet (`cargo test` succeeds trivially).

## Architecture

A **Tauri v2** desktop app for multi-shop 抖音 (Douyin) customer service management. The Rust backend (`src-tauri/`) embeds multiple webviews loaded from `https://fxg.jinritemai.com` (飞鸽客服 platform), injects JS interception scripts, and communicates bidirectionally via Tauri IPC.

### Crate structure (`src-tauri/src/`)

```
main.rs        → entry point, calls lib::run()
lib.rs         → Tauri builder: registers commands, sets up window/webview tree
commands/      → #[tauri::command] handlers invoked from frontend/JS
scripts/       → JS string producers (raw &str injected as initialization_script)
webview/       → webview lifecycle (create, park, activate, delete)
window/        → window resize event → repositions active webview
database/      → SQLite via rusqlite (bundled), auto-migration on first connect
utils/         → HTTP client helpers, protobuf decode, UUID, logger, platform detection
config/        → constants (URLs)
proto/         → .proto file for IM binary protocol, compiled at build time via prost
```

### Webview management ("parking spot" pattern)

The window is split: left 20% = app sidebar (`02_app`), right 80% = content area. Only ONE "08" webview is "active" (full right-area size) at a time. Others are moved to a 1×1 corner position — still visible (`document.visibilityState === "visible"`) so `setInterval`/WebSocket run uninterrupted, avoiding browser throttling.

- `webview/creator.rs` — `create_douyin_webview()`, `create_platform_webview()`, `create_bg_webview()`
  - Each new webview gets: `.initialization_script()` calls for redirect, interception, WS hook
  - Webview IDs: `02_app`, `08pf` (platform homepage), `08bg` (background fallback), `08_douyin_<uuid>` (shop instances)
- `commands/webview_utils.rs` — `activate_08_webview()`, `park_other_08_webviews()`, active webview tracking
- `window/event.rs` — on resize, recalculates active + parked bounds

### JS injection pipeline

Scripts in `src/scripts/` return raw JS strings injected via `WebviewBuilder::initialization_script()`. Execution order:

1. **`redirect.rs`** — SPA route detection & redirect from dead pages to IM workspace; registers Tauri channel for backend→frontend task dispatch
2. **`feige_intercept.rs`** — HTTP request monitor: hooks fetch/XHR/sendBeacon, sends `{type, method, url, headers, body}` to Rust via `on_request`
3. **`ws_hook.rs`** — Wraps `WebSocket` constructor, hooks jinritemai.com sockets, sends connect/open/close/error events + binary messages to Rust
4. **`http_interceptor.rs`** — Response interceptor for specific IM API paths (message history etc.), sends response bytes to Rust
5. **`link_info_interceptor.rs`** — Response interceptor for `get_link_info`, sends parsed JSON to Rust via `on_get_link_info`

### Key data flows

- **Request monitoring**: JS `feige_intercept` → Tauri IPC `on_request` → Rust stores headers/query params in global `SHOP_INFO_PARAMS` / `REQUEST_HEADERS` maps (keyed by webview label)
- **WebSocket messages**: JS `ws_hook` → Tauri IPC `on_ws_binary` (raw bytes) or `on_ws` (events) → protobuf decode via `feige_im_proto()`
- **HTTP response interception**: JS `http_interceptor` / `link_info_interceptor` → Tauri IPC `on_http_response_intercepted` / `on_get_link_info` → protobuf/JSON parse
- **Backend→frontend tasks**: Rust `shop_channel` sends `ShopTask` structs via Tauri Channel → JS `handleBackendTask()` in redirect script

### Protobuf

`proto/dy_im_proto.proto` describes the 抖音 IM binary protocol. `build.rs` runs `protoc_bin_vendored` + `prost_build` to generate Rust code into `OUT_DIR`. The `utils/protobuf.rs` module includes the generated code and provides helpers (`parse_response`, `feige_im_proto`) plus constants for command IDs and sender roles.

### Database

SQLite via `rusqlite` (bundled). Path: `{app_data_dir}/customer_tauri/customer_tauri.db`. Tables: `shop`, `platform`, `task`, `shop_webview`. Migrations run on first connect (`database/migrations.rs`). The `shop_webview` table is the actively used one, mapping webview IDs to shop names.

### Adding a new injected script

1. Create `src/scripts/your_hook.rs` with a `pub fn create_xxx_hook() -> String` returning the JS string
2. Add `pub mod your_hook;` to `src/scripts/mod.rs`
3. Chain `.initialization_script(&scripts::your_hook::create_xxx_hook())` in `webview/creator.rs`
4. If the JS invokes a new Tauri command, add the `#[command]` in `commands/` and register it in `lib.rs`
