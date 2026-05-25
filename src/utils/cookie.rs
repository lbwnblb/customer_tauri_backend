use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub host: String,
    pub path: String,
    pub is_secure: bool,
    pub is_httponly: bool,
    /// Chromium epoch (microseconds since 1601-01-01); 0 means session cookie.
    pub expires_utc: i64,
}

#[derive(Debug)]
pub enum CookieError {
    NotFound(PathBuf),
    Db(rusqlite::Error),
}

impl std::fmt::Display for CookieError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CookieError::NotFound(p) => write!(f, "cookie db not found in {}", p.display()),
            CookieError::Db(e) => write!(f, "cookie db error: {e}"),
        }
    }
}

impl std::error::Error for CookieError {}

impl From<rusqlite::Error> for CookieError {
    fn from(e: rusqlite::Error) -> Self {
        CookieError::Db(e)
    }
}

/// Candidate sub-paths where WebView2 / Chromium stores the Cookies SQLite file.
const COOKIE_SUBPATHS: &[&str] = &[
    "EBWebView/Default/Network/Cookies",
    "EBWebView/Default/Cookies",
    "Default/Network/Cookies",
    "Default/Cookies",
];

fn find_cookie_db(data_dir: &Path) -> Option<PathBuf> {
    COOKIE_SUBPATHS
        .iter()
        .map(|sub| data_dir.join(sub))
        .find(|p| p.exists())
}

/// Read all cookies from the WebView2 data directory.
///
/// Opens the SQLite Cookies file read-only so it can be accessed while the
/// webview is running (WebView2 does not hold an exclusive lock on the file).
///
/// **Note on Windows**: WebView2 stores cookie values encrypted with DPAPI +
/// AES-256-GCM. The `value` column is populated only for non-encrypted entries;
/// for encrypted ones `value` will be an empty string. Decryption requires the
/// `Local State` key and DPAPI, which is outside the scope of this helper.
pub fn get_cookies(data_dir: &Path) -> Result<Vec<Cookie>, CookieError> {
    let db_path = find_cookie_db(data_dir)
        .ok_or_else(|| CookieError::NotFound(data_dir.to_path_buf()))?;

    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    let mut stmt = conn.prepare(
        "SELECT name, value, host_key, path, is_secure, is_httponly, expires_utc \
         FROM cookies",
    )?;

    let cookies = stmt
        .query_map([], |row| {
            Ok(Cookie {
                name: row.get(0)?,
                value: row.get(1)?,
                host: row.get(2)?,
                path: row.get(3)?,
                is_secure: row.get::<_, i32>(4)? != 0,
                is_httponly: row.get::<_, i32>(5)? != 0,
                expires_utc: row.get(6)?,
            })
        })?
        .filter_map(|r| {
            r.map_err(|e| log::warn!("cookie row error: {e}"))
                .ok()
        })
        .collect();

    Ok(cookies)
}

/// Filter cookies by host suffix (e.g. `"jinritemai.com"` matches
/// `"fxg.jinritemai.com"` and `".jinritemai.com"`).
pub fn get_cookies_for_host<'a>(cookies: &'a [Cookie], host: &str) -> Vec<&'a Cookie> {
    cookies
        .iter()
        .filter(|c| {
            let h = c.host.trim_start_matches('.');
            h == host || h.ends_with(&format!(".{host}"))
        })
        .collect()
}

/// Build a `Cookie: name=value; ...` header string for the given host.
/// Only includes cookies whose `value` field is non-empty (i.e. not encrypted).
pub fn cookies_as_header(data_dir: &Path, host: &str) -> Result<String, CookieError> {
    let all = get_cookies(data_dir)?;
    let header = get_cookies_for_host(&all, host)
        .iter()
        .filter(|c| !c.value.is_empty())
        .map(|c| format!("{}={}", c.name, c.value))
        .collect::<Vec<_>>()
        .join("; ");
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::app_data::app_data_dir_home_index;

    fn data_dir() -> PathBuf {
        app_data_dir_home_index()
    }

    /// 打印 data_dir 下所有能读到的 cookie（含加密的 value 为空的条目）
    #[test]
    fn test_print_all_cookies() {
        let dir = data_dir();
        match get_cookies(&dir) {
            Ok(cookies) => {
                println!("共 {} 条 cookie，data_dir={}", cookies.len(), dir.display());
                for c in &cookies {
                    println!(
                        "  host={:30} name={:30} value={:?} expires={}",
                        c.host, c.name,
                        if c.value.is_empty() { "<encrypted>" } else { &c.value },
                        c.expires_utc
                    );
                }
            }
            Err(e) => println!("读取失败: {e}  (data_dir={})", dir.display()),
        }
        let token = get_cookies(&app_data_dir_home_index())
            .ok()
            .and_then(|cookies| {
                cookies.into_iter()
                    .find(|c| c.name == "Authorization")
                    .map(|c| c.value)
            })
            .filter(|v| !v.is_empty());

        println!("Authorization cookie: {token:?}")

    }

    /// 打印 Authorization cookie 的值，验证登录后能否读到
    #[test]
    fn test_read_authorization_cookie() {
        let dir = data_dir();
        let token = get_cookies(&dir)
            .ok()
            .and_then(|cookies| {
                cookies.into_iter()
                    .find(|c| c.name == "Authorization")
                    .map(|c| c.value)
            });
        match token {
            Some(v) if !v.is_empty() => println!("Authorization cookie: {v}"),
            Some(_) => println!("Authorization cookie 存在但 value 为空（已加密）"),
            None => println!("未找到 Authorization cookie"),
        }
    }

    /// 打印 data_dir 下能找到的 Cookies 文件路径，确认路径探测是否正确
    #[test]
    fn test_find_cookie_db_path() {
        let dir = data_dir();
        match find_cookie_db(&dir) {
            Some(p) => println!("Cookies 文件: {}", p.display()),
            None => {
                println!("未找到 Cookies 文件，候选路径如下：");
                for sub in COOKIE_SUBPATHS {
                    let p = dir.join(sub);
                    println!("  {} — exists={}", p.display(), p.exists());
                }
            }
        }
    }

    /// 构造指定 host 的 Cookie header 字符串
    #[test]
    fn test_cookies_as_header_for_tauri_localhost() {
        let dir = data_dir();
        match cookies_as_header(&dir, "tauri.localhost") {
            Ok(h) if h.is_empty() => println!("tauri.localhost 无明文 cookie"),
            Ok(h) => println!("Cookie header: {h}"),
            Err(e) => println!("失败: {e}"),
        }
    }
}
