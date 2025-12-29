use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct HuntDatabase {
    pub conn: Connection,
}

impl HuntDatabase {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Enable WAL mode for better concurrency/safety
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
        
        // Init tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS evidence (
                id INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                file_path TEXT NOT NULL,
                encrypted_key_nonce BLOB NOT NULL, -- The nonce used to encrypt this specific file
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        Ok(Self { conn })
    }

    pub fn insert_evidence(&self, desc: &str, file_path: &str, nonce: &[u8]) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO evidence (description, file_path, encrypted_key_nonce) VALUES (?1, ?2, ?3)",
            params![desc, file_path, nonce],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
}
