use serde::Serialize;
use crate::database::get_connection;

#[derive(Serialize)]
pub struct ShopPlatformId {
    pub shop_name: String,
    pub platform_shop_id: String,
}

#[tauri::command]
pub fn get_shop_platform_ids() -> Vec<ShopPlatformId> {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut stmt = match conn.prepare(
        "SELECT sw.shop_name, ws.platform_shop_id
         FROM shop_webview sw
         INNER JOIN webview_shop_id ws ON sw.id = ws.webview_id",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    stmt.query_map([], |row| {
        Ok(ShopPlatformId {
            shop_name: row.get(0)?,
            platform_shop_id: row.get(1)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}
