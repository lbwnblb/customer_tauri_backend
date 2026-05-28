use super::connection::get_connection;

fn create_table(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_reply (
            shop_id TEXT PRIMARY KEY,
            enabled INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )?;
    Ok(())
}

pub fn set_ai_reply_enabled(shop_id: &str, enabled: bool) -> Result<(), rusqlite::Error> {
    let conn = get_connection()?;
    create_table(&conn)?;
    conn.execute(
        "INSERT INTO ai_reply (shop_id, enabled) VALUES (?1, ?2)
         ON CONFLICT(shop_id) DO UPDATE SET enabled = excluded.enabled",
        rusqlite::params![shop_id, enabled as i32],
    )?;
    Ok(())
}

pub fn get_ai_reply_enabled(shop_id: &str) -> Result<bool, rusqlite::Error> {
    let conn = get_connection()?;
    create_table(&conn)?;
    let result = conn.query_row(
        "SELECT enabled FROM ai_reply WHERE shop_id = ?1",
        [shop_id],
        |row| row.get::<_, i32>(0),
    );
    match result {
        Ok(v) => Ok(v != 0),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(true), // 默认开启
        Err(e) => Err(e),
    }
}

pub fn get_all_ai_reply() -> Result<Vec<(String, bool)>, rusqlite::Error> {
    let conn = get_connection()?;
    create_table(&conn)?;
    let mut stmt = conn.prepare("SELECT shop_id, enabled FROM ai_reply")?;
    let iter = stmt.query_map([], |row| {
        let shop_id: String = row.get(0)?;
        let enabled: i32 = row.get(1)?;
        Ok((shop_id, enabled != 0))
    })?;
    iter.collect()
}
