use std::fs;
use std::path::Path;
use std::sync::Mutex;

use log::info;
use rusqlite::Connection;

use super::error::DbResult;
use super::migration::run_migrations;

/// Thread-safe database wrapper for use as Tauri managed state.
#[allow(dead_code)]
pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// Opens (or creates) the SQLite database, configures pragmas, and runs migrations.
    ///
    /// # Arguments
    /// * `app_data_dir` - Directory where `contentjuiceos.db` will be stored.
    ///
    /// # Panics
    /// This function does not panic, but callers should treat errors as fatal
    /// since the app cannot function without a database.
    pub fn initialize(app_data_dir: &Path) -> DbResult<Self> {
        fs::create_dir_all(app_data_dir)?;

        let db_path = app_data_dir.join("contentjuiceos.db");
        info!("Opening database at {}", db_path.display());

        let conn = Connection::open(&db_path)?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        run_migrations(&conn)?;

        info!("Database initialized successfully");

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn initialize_creates_db_file_with_wal_mode() {
        let tmp = env::temp_dir().join("contentjuiceos_test_db_init");
        let _ = fs::remove_dir_all(&tmp);

        let db = Database::initialize(&tmp).unwrap();

        let db_path = tmp.join("contentjuiceos.db");
        assert!(db_path.exists(), "Database file should exist");

        let conn = db.conn.lock().unwrap();
        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_lowercase(), "wal");

        let fk: i32 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1);

        drop(conn);
        drop(db);
        let _ = fs::remove_dir_all(&tmp);
    }
}
