use super::connection::get_connection;

pub struct WebviewShopId {
    pub webview_id: String,
    pub platform_shop_id: String,
}

fn create_table(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS webview_shop_id (
            webview_id TEXT PRIMARY KEY,
            platform_shop_id TEXT NOT NULL DEFAULT ''
        )",
        [],
    )?;
    Ok(())
}

pub fn upsert_webview_shop_id(webview_id: &str, platform_shop_id: &str) -> Result<(), rusqlite::Error> {
    let conn = get_connection()?;
    create_table(&conn)?;
    conn.execute(
        "INSERT INTO webview_shop_id (webview_id, platform_shop_id) VALUES (?1, ?2)
         ON CONFLICT(webview_id) DO UPDATE SET platform_shop_id = excluded.platform_shop_id",
        [webview_id, platform_shop_id],
    )?;
    Ok(())
}

pub fn get_platform_shop_id(webview_id: &str) -> Result<Option<String>, rusqlite::Error> {
    let conn = get_connection()?;
    create_table(&conn)?;
    let mut stmt = conn.prepare(
        "SELECT platform_shop_id FROM webview_shop_id WHERE webview_id = ?1",
    )?;
    let mut rows = stmt.query([webview_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn delete_webview_shop_id(webview_id: &str) -> Result<(), rusqlite::Error> {
    let conn = get_connection()?;
    create_table(&conn)?;
    conn.execute(
        "DELETE FROM webview_shop_id WHERE webview_id = ?1",
        [webview_id],
    )?;
    Ok(())
}
