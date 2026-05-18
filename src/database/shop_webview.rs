use super::connection::get_connection;

#[derive(Debug)]
pub struct ShopWebview {
    pub id: String,
    pub shop_name: String,
}

fn create_table(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS shop_webview (
            id TEXT PRIMARY KEY,
            shop_name TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        )",
        [],
    )?;
    Ok(())
}

pub fn save_shop_webview(id: &str, shop_name: &str) -> Result<(), rusqlite::Error> {
    let conn = get_connection()?;
    create_table(&conn)?;
    conn.execute(
        "INSERT INTO shop_webview (id, shop_name) VALUES (?1, ?2)",
        [id, shop_name],
    )?;
    Ok(())
}

pub fn delete_shop_webview(id: &str) -> Result<(), rusqlite::Error> {
    let conn = get_connection()?;
    create_table(&conn)?;
    conn.execute("DELETE FROM shop_webview WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get_all_shops() -> Result<Vec<ShopWebview>, rusqlite::Error> {
    let conn = get_connection()?;
    create_table(&conn)?;
    let mut stmt = conn.prepare("SELECT id, shop_name FROM shop_webview")?;
    let shop_iter = stmt.query_map([], |row| {
        Ok(ShopWebview {
            id: row.get(0)?,
            shop_name: row.get(1)?,
        })
    })?;
    
    let mut shops = Vec::new();
    for shop in shop_iter {
        shops.push(shop?);
    }
    Ok(shops)
}
