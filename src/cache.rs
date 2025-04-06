use std::sync::Mutex;

use rusqlite::{Connection, Result};

pub struct Cache {
    conn: Mutex<Connection>,
}

impl Cache {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Cache {
            conn: Mutex::new(conn),
        })
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO cache (key, value) VALUES (?1, ?2)",
            &[key, value],
        )?;
        Ok(())
    }
}
