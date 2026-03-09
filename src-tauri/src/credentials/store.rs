use std::sync::Arc;

use log::{info, warn};

use crate::db::Database;

use super::encryption;
use super::error::{CredResult, CredentialError};
use super::types::{credential_key, CredentialBackend, CredentialKind, OAuthTokens};

/// Trait for pluggable credential storage backends.
pub trait CredentialStore: Send + Sync {
    fn store(&self, key: &str, value: &str) -> CredResult<()>;
    fn retrieve(&self, key: &str) -> CredResult<Option<String>>;
    fn delete(&self, key: &str) -> CredResult<()>;
    fn exists(&self, key: &str) -> CredResult<bool>;
}

// ---------------------------------------------------------------------------
// KeychainStore
// ---------------------------------------------------------------------------

pub struct KeychainStore;

impl KeychainStore {
    const SERVICE: &'static str = "com.contentjuiceos.app";

    /// Probe the OS keychain by writing and deleting a test entry.
    /// Returns `Ok(Self)` if the keychain is usable, `Err` otherwise.
    pub fn probe() -> CredResult<Self> {
        let entry = keyring::Entry::new(Self::SERVICE, "__contentjuiceos_probe__")
            .map_err(|e| CredentialError::Keychain(format!("Keychain probe init failed: {e}")))?;

        entry
            .set_password("probe")
            .map_err(|e| CredentialError::Keychain(format!("Keychain probe write failed: {e}")))?;

        entry
            .delete_credential()
            .map_err(|e| CredentialError::Keychain(format!("Keychain probe delete failed: {e}")))?;

        Ok(Self)
    }
}

impl CredentialStore for KeychainStore {
    fn store(&self, key: &str, value: &str) -> CredResult<()> {
        let entry = keyring::Entry::new(Self::SERVICE, key).map_err(|e| {
            CredentialError::Keychain(format!("Keychain entry creation failed: {e}"))
        })?;
        entry
            .set_password(value)
            .map_err(|e| CredentialError::Keychain(format!("Keychain store failed: {e}")))?;
        Ok(())
    }

    fn retrieve(&self, key: &str) -> CredResult<Option<String>> {
        let entry = keyring::Entry::new(Self::SERVICE, key).map_err(|e| {
            CredentialError::Keychain(format!("Keychain entry creation failed: {e}"))
        })?;
        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(CredentialError::Keychain(format!(
                "Keychain retrieve failed: {e}"
            ))),
        }
    }

    fn delete(&self, key: &str) -> CredResult<()> {
        let entry = keyring::Entry::new(Self::SERVICE, key).map_err(|e| {
            CredentialError::Keychain(format!("Keychain entry creation failed: {e}"))
        })?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // Already gone — not an error
            Err(e) => Err(CredentialError::Keychain(format!(
                "Keychain delete failed: {e}"
            ))),
        }
    }

    fn exists(&self, key: &str) -> CredResult<bool> {
        let entry = keyring::Entry::new(Self::SERVICE, key).map_err(|e| {
            CredentialError::Keychain(format!("Keychain entry creation failed: {e}"))
        })?;
        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(CredentialError::Keychain(format!(
                "Keychain exists check failed: {e}"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// EncryptedSqliteStore
// ---------------------------------------------------------------------------

pub struct EncryptedSqliteStore {
    database: Arc<Database>,
    key: [u8; 32],
}

impl EncryptedSqliteStore {
    pub fn new(database: Arc<Database>) -> CredResult<Self> {
        let key = encryption::derive_key()?;
        Ok(Self { database, key })
    }

    #[cfg(test)]
    pub fn new_with_key(database: Arc<Database>, key: [u8; 32]) -> Self {
        Self { database, key }
    }
}

impl CredentialStore for EncryptedSqliteStore {
    fn store(&self, key: &str, value: &str) -> CredResult<()> {
        let encrypted = encryption::encrypt(&self.key, value)?;
        let conn = self
            .database
            .conn
            .lock()
            .map_err(|e| CredentialError::Encryption(format!("Database lock failed: {e}")))?;
        conn.execute(
            "INSERT OR REPLACE INTO secure_credentials (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![key, encrypted],
        ).map_err(|e| CredentialError::Database(e.into()))?;
        Ok(())
    }

    fn retrieve(&self, key: &str) -> CredResult<Option<String>> {
        let conn = self
            .database
            .conn
            .lock()
            .map_err(|e| CredentialError::Encryption(format!("Database lock failed: {e}")))?;
        let result: Option<String> = conn
            .query_row(
                "SELECT value FROM secure_credentials WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .ok();
        match result {
            Some(encrypted) => Ok(Some(encryption::decrypt(&self.key, &encrypted)?)),
            None => Ok(None),
        }
    }

    fn delete(&self, key: &str) -> CredResult<()> {
        let conn = self
            .database
            .conn
            .lock()
            .map_err(|e| CredentialError::Encryption(format!("Database lock failed: {e}")))?;
        conn.execute("DELETE FROM secure_credentials WHERE key = ?1", [key])
            .map_err(|e| CredentialError::Database(e.into()))?;
        Ok(())
    }

    fn exists(&self, key: &str) -> CredResult<bool> {
        let conn = self
            .database
            .conn
            .lock()
            .map_err(|e| CredentialError::Encryption(format!("Database lock failed: {e}")))?;
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM secure_credentials WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .map_err(|e| CredentialError::Database(e.into()))?;
        Ok(count > 0)
    }
}

// ---------------------------------------------------------------------------
// CredentialManager — public API, selects backend on init
// ---------------------------------------------------------------------------

pub struct CredentialManager {
    store: Box<dyn CredentialStore>,
    backend: CredentialBackend,
}

impl CredentialManager {
    /// Probes the OS keychain. Falls back to encrypted SQLite if keychain is unavailable.
    pub fn initialize(database: Arc<Database>) -> Self {
        match KeychainStore::probe() {
            Ok(keychain) => {
                info!("Credential storage: using OS keychain");
                Self {
                    store: Box::new(keychain),
                    backend: CredentialBackend::Keychain,
                }
            }
            Err(e) => {
                warn!("OS keychain unavailable ({e}), falling back to encrypted SQLite storage");
                let sqlite_store = EncryptedSqliteStore::new(database)
                    .expect("Failed to initialize encrypted SQLite credential store");
                Self {
                    store: Box::new(sqlite_store),
                    backend: CredentialBackend::EncryptedSqlite,
                }
            }
        }
    }

    /// Returns which backend is active.
    pub fn backend(&self) -> CredentialBackend {
        self.backend
    }

    /// Store a raw credential value.
    pub fn store_credential(&self, kind: &CredentialKind, value: &str) -> CredResult<()> {
        let key = credential_key(kind);
        self.store.store(&key, value)
    }

    /// Retrieve a raw credential value.
    pub fn get_credential(&self, kind: &CredentialKind) -> CredResult<Option<String>> {
        let key = credential_key(kind);
        self.store.retrieve(&key)
    }

    /// Delete a credential.
    pub fn delete_credential(&self, kind: &CredentialKind) -> CredResult<()> {
        let key = credential_key(kind);
        self.store.delete(&key)
    }

    /// Check if a credential exists.
    pub fn has_credential(&self, kind: &CredentialKind) -> CredResult<bool> {
        let key = credential_key(kind);
        self.store.exists(&key)
    }

    /// Convenience: store OAuth tokens as a JSON blob.
    pub fn store_platform_tokens(
        &self,
        connection_id: &str,
        tokens: &OAuthTokens,
    ) -> CredResult<()> {
        let kind = CredentialKind::PlatformToken {
            connection_id: connection_id.to_string(),
        };
        let json = serde_json::to_string(tokens)?;
        self.store_credential(&kind, &json)
    }

    /// Convenience: retrieve OAuth tokens.
    pub fn get_platform_tokens(&self, connection_id: &str) -> CredResult<Option<OAuthTokens>> {
        let kind = CredentialKind::PlatformToken {
            connection_id: connection_id.to_string(),
        };
        match self.get_credential(&kind)? {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::run_migrations;
    use rusqlite::Connection;
    use std::sync::Mutex;

    /// Create an in-memory Database with migrations applied (including V2).
    fn test_database() -> Arc<Database> {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        Arc::new(Database {
            conn: Mutex::new(conn),
        })
    }

    fn test_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        key[0] = 0x42;
        key[31] = 0xFF;
        key
    }

    #[test]
    fn encrypted_sqlite_store_crud() {
        let db = test_database();
        let store = EncryptedSqliteStore::new_with_key(db, test_key());

        // Store
        store.store("test:key1", "secret_value").unwrap();

        // Exists
        assert!(store.exists("test:key1").unwrap());
        assert!(!store.exists("test:nonexistent").unwrap());

        // Retrieve
        let value = store.retrieve("test:key1").unwrap();
        assert_eq!(value.as_deref(), Some("secret_value"));

        // Retrieve nonexistent
        let missing = store.retrieve("test:nonexistent").unwrap();
        assert!(missing.is_none());

        // Delete
        store.delete("test:key1").unwrap();
        assert!(!store.exists("test:key1").unwrap());

        // Delete nonexistent is not an error
        store.delete("test:nonexistent").unwrap();
    }

    #[test]
    fn encrypted_sqlite_store_overwrites() {
        let db = test_database();
        let store = EncryptedSqliteStore::new_with_key(db, test_key());

        store.store("test:overwrite", "value1").unwrap();
        store.store("test:overwrite", "value2").unwrap();

        let value = store.retrieve("test:overwrite").unwrap();
        assert_eq!(value.as_deref(), Some("value2"));
    }

    #[test]
    fn encrypted_sqlite_values_are_not_plaintext() {
        let db = test_database();
        let store = EncryptedSqliteStore::new_with_key(db.clone(), test_key());

        store.store("test:encrypted", "my_secret").unwrap();

        // Read raw value from DB — it should NOT be "my_secret"
        let conn = db.conn.lock().unwrap();
        let raw: String = conn
            .query_row(
                "SELECT value FROM secure_credentials WHERE key = 'test:encrypted'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_ne!(raw, "my_secret");
        assert!(raw.len() > "my_secret".len()); // Encrypted + base64 is longer
    }
}
