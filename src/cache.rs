#![allow(dead_code)]

use std::sync::LazyLock;

use rusqlite::Connection;

const CACHE_PATH: &str = "cache.dat";
const CACHE_SCHEMA: &str = include_str!("../cache.sql");

static CACHE_VERSION: LazyLock<String> =
    LazyLock::new(|| blake3::hash(CACHE_SCHEMA.as_bytes()).to_hex().to_string());

pub struct Cache {
    conn: Connection,
}

impl Cache {
    pub fn new() -> Self {
        let conn = Connection::open(CACHE_PATH).unwrap();
        conn.execute(CACHE_SCHEMA, []).unwrap();
        Self { conn }
    }
}
