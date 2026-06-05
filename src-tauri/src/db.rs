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
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                description TEXT NOT NULL,
                file_path TEXT NOT NULL,
                encrypted_key_nonce BLOB NOT NULL, -- The nonce used to encrypt this specific file
                sha256_hash TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Ensure sha256_hash column is added if database already existed
        let _ = conn.execute("ALTER TABLE evidence ADD COLUMN sha256_hash TEXT", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                event_date TEXT NOT NULL,
                event_type TEXT DEFAULT 'other',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS parties (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                role TEXT DEFAULT 'Other',
                email TEXT,
                phone TEXT,
                notes TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS complaint_sections (
                section_id TEXT PRIMARY KEY,
                content TEXT,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        Ok(Self { conn })
    }

    pub fn insert_evidence(&self, desc: &str, file_path: &str, nonce: &[u8], sha256_hash: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO evidence (description, file_path, encrypted_key_nonce, sha256_hash) VALUES (?1, ?2, ?3, ?4)",
            params![desc, file_path, nonce, sha256_hash],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
}
