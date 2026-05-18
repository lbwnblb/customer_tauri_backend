use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS shop (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            platform TEXT NOT NULL,
            account TEXT NOT NULL DEFAULT '',
            password TEXT NOT NULL DEFAULT '',
            cookies TEXT NOT NULL DEFAULT '',
            status INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS platform (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            url TEXT NOT NULL DEFAULT '',
            status INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS task (
            id TEXT PRIMARY KEY,
            shop_id TEXT NOT NULL,
            task_type TEXT NOT NULL DEFAULT '',
            status INTEGER NOT NULL DEFAULT 0,
            result TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (shop_id) REFERENCES shop(id)
        );

        CREATE TABLE IF NOT EXISTS shop_webview (
            id TEXT PRIMARY KEY,
            shop_name TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        ",
    )?;

    Ok(())
}
