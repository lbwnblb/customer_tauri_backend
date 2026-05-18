use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use super::migrations;
use crate::utils::app_data_dir;

static INITIALIZED: Mutex<bool> = Mutex::new(false);

fn get_db_path() -> PathBuf {
    let base = app_data_dir();
    PathBuf::from(base).join("customer_tauri").join("customer_tauri.db")
}

pub fn get_connection() -> Result<Connection, rusqlite::Error> {
    let path = get_db_path();
    let mut needs_init = false;

    {
        let mut initialized = INITIALIZED.lock().unwrap();
        if !*initialized {
            *initialized = true;
            needs_init = true;
        }
    }

    if needs_init {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                let mut initialized = INITIALIZED.lock().unwrap();
                *initialized = false;
                rusqlite::Error::InvalidParameterName(format!("无法创建数据库目录: {}", e))
            })?;
        }
    }

    let conn = Connection::open(&path)?;

    if needs_init {
        if let Err(e) = migrations::run_migrations(&conn) {
            let mut initialized = INITIALIZED.lock().unwrap();
            *initialized = false;
            return Err(e);
        }
    }

    Ok(conn)
}
