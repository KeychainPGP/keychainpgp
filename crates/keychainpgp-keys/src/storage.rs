use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

use keychainpgp_core::types::TrustLevel;

use crate::error::Result;

/// A record representing a public key stored in the database.
#[derive(Debug, Clone)]
pub struct KeyRecord {
    /// Primary key fingerprint (hex string).
    pub fingerprint: String,
    /// Primary User ID name.
    pub name: Option<String>,
    /// Primary User ID email.
    pub email: Option<String>,
    /// Key algorithm description.
    pub algorithm: String,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
    /// Expiration timestamp (ISO 8601), if any.
    pub expires_at: Option<String>,
    /// Trust level.
    pub trust_level: i32,
    /// Whether this is the user's own key (has a corresponding private key).
    pub is_own_key: bool,
    /// Raw ASCII-armored public key data.
    pub pgp_data: Vec<u8>,
}

/// SQLite-backed storage for public keys.
pub struct KeyStorage {
    conn: Connection,
}

impl KeyStorage {
    /// Open or create the key storage database at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.initialize()?;
        Ok(storage)
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn };
        storage.initialize()?;
        Ok(storage)
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS keys (
                fingerprint TEXT PRIMARY KEY NOT NULL,
                name        TEXT,
                email       TEXT,
                algorithm   TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                expires_at  TEXT,
                trust_level INTEGER NOT NULL DEFAULT 0,
                is_own_key  INTEGER NOT NULL DEFAULT 0,
                pgp_data    BLOB NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_keys_email ON keys(email);
            CREATE INDEX IF NOT EXISTS idx_keys_name  ON keys(name);",
        )?;
        Ok(())
    }

    /// Insert a key record. Returns an error if the fingerprint already exists.
    pub fn insert(&self, record: &KeyRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO keys (fingerprint, name, email, algorithm, created_at, expires_at, trust_level, is_own_key, pgp_data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                record.fingerprint,
                record.name,
                record.email,
                record.algorithm,
                record.created_at,
                record.expires_at,
                record.trust_level,
                record.is_own_key,
                record.pgp_data,
            ],
        )?;
        Ok(())
    }

    /// Get a key record by fingerprint.
    pub fn get(&self, fingerprint: &str) -> Result<Option<KeyRecord>> {
        let record = self
            .conn
            .query_row(
                "SELECT fingerprint, name, email, algorithm, created_at, expires_at, trust_level, is_own_key, pgp_data
                 FROM keys WHERE fingerprint = ?1",
                params![fingerprint],
                |row| {
                    Ok(KeyRecord {
                        fingerprint: row.get(0)?,
                        name: row.get(1)?,
                        email: row.get(2)?,
                        algorithm: row.get(3)?,
                        created_at: row.get(4)?,
                        expires_at: row.get(5)?,
                        trust_level: row.get(6)?,
                        is_own_key: row.get(7)?,
                        pgp_data: row.get(8)?,
                    })
                },
            )
            .optional()?;
        Ok(record)
    }

    /// List all key records.
    pub fn list_all(&self) -> Result<Vec<KeyRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT fingerprint, name, email, algorithm, created_at, expires_at, trust_level, is_own_key, pgp_data
             FROM keys ORDER BY is_own_key DESC, name ASC",
        )?;
        let records = stmt
            .query_map([], |row| {
                Ok(KeyRecord {
                    fingerprint: row.get(0)?,
                    name: row.get(1)?,
                    email: row.get(2)?,
                    algorithm: row.get(3)?,
                    created_at: row.get(4)?,
                    expires_at: row.get(5)?,
                    trust_level: row.get(6)?,
                    is_own_key: row.get(7)?,
                    pgp_data: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(records)
    }

    /// Search keys by name or email (case-insensitive partial match).
    pub fn search(&self, query: &str) -> Result<Vec<KeyRecord>> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT fingerprint, name, email, algorithm, created_at, expires_at, trust_level, is_own_key, pgp_data
             FROM keys
             WHERE name LIKE ?1 COLLATE NOCASE
                OR email LIKE ?1 COLLATE NOCASE
                OR fingerprint LIKE ?1 COLLATE NOCASE
             ORDER BY is_own_key DESC, name ASC",
        )?;
        let records = stmt
            .query_map(params![pattern], |row| {
                Ok(KeyRecord {
                    fingerprint: row.get(0)?,
                    name: row.get(1)?,
                    email: row.get(2)?,
                    algorithm: row.get(3)?,
                    created_at: row.get(4)?,
                    expires_at: row.get(5)?,
                    trust_level: row.get(6)?,
                    is_own_key: row.get(7)?,
                    pgp_data: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(records)
    }

    /// Delete a key by fingerprint.
    pub fn delete(&self, fingerprint: &str) -> Result<bool> {
        let rows = self
            .conn
            .execute("DELETE FROM keys WHERE fingerprint = ?1", params![fingerprint])?;
        Ok(rows > 0)
    }

    /// Update the trust level for a key.
    pub fn set_trust(&self, fingerprint: &str, trust_level: TrustLevel) -> Result<bool> {
        let level: i32 = match trust_level {
            TrustLevel::Unknown => 0,
            TrustLevel::Unverified => 1,
            TrustLevel::Verified => 2,
        };
        let rows = self.conn.execute(
            "UPDATE keys SET trust_level = ?1 WHERE fingerprint = ?2",
            params![level, fingerprint],
        )?;
        Ok(rows > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(fingerprint: &str, name: &str, email: &str) -> KeyRecord {
        KeyRecord {
            fingerprint: fingerprint.to_string(),
            name: Some(name.to_string()),
            email: Some(email.to_string()),
            algorithm: "Ed25519".to_string(),
            created_at: "2026-02-20T00:00:00Z".to_string(),
            expires_at: Some("2028-02-20T00:00:00Z".to_string()),
            trust_level: 0,
            is_own_key: false,
            pgp_data: b"fake-pgp-data".to_vec(),
        }
    }

    #[test]
    fn test_insert_and_get() {
        let storage = KeyStorage::open_in_memory().unwrap();
        let record = make_record("AAAA1111", "Alice", "alice@example.com");
        storage.insert(&record).unwrap();

        let fetched = storage.get("AAAA1111").unwrap().unwrap();
        assert_eq!(fetched.name.as_deref(), Some("Alice"));
        assert_eq!(fetched.email.as_deref(), Some("alice@example.com"));
    }

    #[test]
    fn test_search_by_email() {
        let storage = KeyStorage::open_in_memory().unwrap();
        storage.insert(&make_record("AAAA", "Alice", "alice@example.com")).unwrap();
        storage.insert(&make_record("BBBB", "Bob", "bob@example.com")).unwrap();

        let results = storage.search("alice").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].fingerprint, "AAAA");
    }

    #[test]
    fn test_delete() {
        let storage = KeyStorage::open_in_memory().unwrap();
        storage.insert(&make_record("AAAA", "Alice", "alice@example.com")).unwrap();
        assert!(storage.delete("AAAA").unwrap());
        assert!(storage.get("AAAA").unwrap().is_none());
    }
}
