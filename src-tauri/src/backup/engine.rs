use std::fs;
use std::path::Path;

use chrono::Local;
use rusqlite::backup::Backup;
use rusqlite::Connection;
use serde::Serialize;

use crate::db::error::{DbError, DbResult};

const BACKUP_PREFIX: &str = "contentjuiceos_backup_";
const BACKUP_EXT: &str = ".db";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub filename: String,
    pub created_at: String,
    pub size_bytes: u64,
}

pub fn create_backup(conn: &Connection, backup_dir: &Path) -> DbResult<BackupInfo> {
    fs::create_dir_all(backup_dir)?;

    let now = Local::now();
    let filename = format!(
        "{}{}{BACKUP_EXT}",
        BACKUP_PREFIX,
        now.format("%Y%m%d_%H%M%S")
    );
    let backup_path = backup_dir.join(&filename);

    let mut dest = Connection::open(&backup_path)?;
    let backup = Backup::new(conn, &mut dest)?;
    backup.step(-1)?;

    let metadata = fs::metadata(&backup_path)?;

    Ok(BackupInfo {
        filename,
        created_at: now.to_rfc3339(),
        size_bytes: metadata.len(),
    })
}

pub fn create_prerestore_backup(conn: &Connection, backup_dir: &Path) -> DbResult<BackupInfo> {
    fs::create_dir_all(backup_dir)?;

    let now = Local::now();
    let filename = format!(
        "_prerestore_{}{}{BACKUP_EXT}",
        BACKUP_PREFIX,
        now.format("%Y%m%d_%H%M%S")
    );
    let backup_path = backup_dir.join(&filename);

    let mut dest = Connection::open(&backup_path)?;
    let backup = Backup::new(conn, &mut dest)?;
    backup.step(-1)?;

    let metadata = fs::metadata(&backup_path)?;

    Ok(BackupInfo {
        filename,
        created_at: now.to_rfc3339(),
        size_bytes: metadata.len(),
    })
}

pub fn list_backups(backup_dir: &Path) -> DbResult<Vec<BackupInfo>> {
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(backup_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if name.ends_with(BACKUP_EXT) && name.contains(BACKUP_PREFIX) {
            let metadata = entry.metadata()?;
            let created_at = metadata
                .modified()
                .map(|t| {
                    let dt: chrono::DateTime<Local> = t.into();
                    dt.to_rfc3339()
                })
                .unwrap_or_default();

            backups.push(BackupInfo {
                filename: name,
                created_at,
                size_bytes: metadata.len(),
            });
        }
    }

    // Newest first (filenames contain timestamps, so reverse sort works)
    backups.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(backups)
}

pub fn cleanup_old_backups(backup_dir: &Path, max_backups: u32) -> DbResult<u32> {
    let backups = list_backups(backup_dir)?;
    let max = max_backups as usize;
    let mut deleted = 0u32;

    if backups.len() > max {
        for backup in &backups[max..] {
            let path = backup_dir.join(&backup.filename);
            fs::remove_file(path)?;
            deleted += 1;
        }
    }

    Ok(deleted)
}

fn validate_filename(filename: &str) -> DbResult<()> {
    if filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
        || !filename.contains(BACKUP_PREFIX)
        || !filename.ends_with(BACKUP_EXT)
    {
        return Err(DbError::Backup(format!(
            "Invalid backup filename: {filename}"
        )));
    }
    Ok(())
}

pub fn restore_backup(conn: &mut Connection, backup_dir: &Path, filename: &str) -> DbResult<()> {
    validate_filename(filename)?;

    let backup_path = backup_dir.join(filename);
    if !backup_path.exists() {
        return Err(DbError::Backup(format!(
            "Backup file not found: {filename}"
        )));
    }

    let source = Connection::open(&backup_path)?;
    let backup = Backup::new(&source, conn)?;
    backup.step(-1)?;
    drop(backup);
    drop(source);

    // Checkpoint WAL after restore
    conn.pragma_update(None, "wal_checkpoint", "TRUNCATE")?;

    Ok(())
}

pub fn delete_backup(backup_dir: &Path, filename: &str) -> DbResult<()> {
    validate_filename(filename)?;

    let path = backup_dir.join(filename);
    if !path.exists() {
        return Err(DbError::Backup(format!(
            "Backup file not found: {filename}"
        )));
    }

    fs::remove_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::run_migrations;
    use std::env;

    fn setup_test_db(dir: &Path) -> Connection {
        fs::create_dir_all(dir).unwrap();
        let db_path = dir.join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn test_dir(name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!("contentjuiceos_backup_test_{name}"))
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn create_backup_produces_valid_file() {
        let dir = test_dir("create");
        cleanup(&dir);
        let conn = setup_test_db(&dir);
        let backup_dir = dir.join("backups");

        let info = create_backup(&conn, &backup_dir).unwrap();

        assert!(info.filename.starts_with(BACKUP_PREFIX));
        assert!(info.filename.ends_with(BACKUP_EXT));
        assert!(info.size_bytes > 0);
        assert!(backup_dir.join(&info.filename).exists());

        // Verify it's a valid SQLite database
        let dest = Connection::open(backup_dir.join(&info.filename)).unwrap();
        let count: u32 = dest
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count > 0);

        cleanup(&dir);
    }

    #[test]
    fn list_backups_returns_newest_first() {
        let dir = test_dir("list");
        cleanup(&dir);
        let conn = setup_test_db(&dir);
        let backup_dir = dir.join("backups");

        let info1 = create_backup(&conn, &backup_dir).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let info2 = create_backup(&conn, &backup_dir).unwrap();

        let backups = list_backups(&backup_dir).unwrap();
        assert_eq!(backups.len(), 2);
        assert_eq!(backups[0].filename, info2.filename);
        assert_eq!(backups[1].filename, info1.filename);

        cleanup(&dir);
    }

    #[test]
    fn cleanup_deletes_oldest() {
        let dir = test_dir("cleanup");
        cleanup(&dir);
        let conn = setup_test_db(&dir);
        let backup_dir = dir.join("backups");

        for _ in 0..5 {
            create_backup(&conn, &backup_dir).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        let deleted = cleanup_old_backups(&backup_dir, 3).unwrap();
        assert_eq!(deleted, 2);

        let remaining = list_backups(&backup_dir).unwrap();
        assert_eq!(remaining.len(), 3);

        cleanup(&dir);
    }

    #[test]
    fn restore_recovers_data() {
        let dir = test_dir("restore");
        cleanup(&dir);
        let mut conn = setup_test_db(&dir);
        let backup_dir = dir.join("backups");

        // Insert test data
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('test.key', 'original')",
            [],
        )
        .unwrap();

        // Create backup
        let info = create_backup(&conn, &backup_dir).unwrap();

        // Delete the data
        conn.execute("DELETE FROM settings WHERE key = 'test.key'", [])
            .unwrap();
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM settings WHERE key = 'test.key'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);

        // Restore
        restore_backup(&mut conn, &backup_dir, &info.filename).unwrap();

        // Verify data is back
        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'test.key'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "original");

        cleanup(&dir);
    }

    #[test]
    fn path_traversal_rejected() {
        let dir = test_dir("traversal");
        cleanup(&dir);
        let mut conn = setup_test_db(&dir);
        let backup_dir = dir.join("backups");

        assert!(restore_backup(&mut conn, &backup_dir, "../evil.db").is_err());
        assert!(restore_backup(&mut conn, &backup_dir, "..\\evil.db").is_err());
        assert!(restore_backup(&mut conn, &backup_dir, "foo/bar.db").is_err());
        assert!(delete_backup(&backup_dir, "../evil.db").is_err());

        cleanup(&dir);
    }

    #[test]
    fn delete_backup_works() {
        let dir = test_dir("delete");
        cleanup(&dir);
        let conn = setup_test_db(&dir);
        let backup_dir = dir.join("backups");

        let info = create_backup(&conn, &backup_dir).unwrap();
        assert!(backup_dir.join(&info.filename).exists());

        delete_backup(&backup_dir, &info.filename).unwrap();
        assert!(!backup_dir.join(&info.filename).exists());

        cleanup(&dir);
    }

    #[test]
    fn list_backups_empty_dir() {
        let dir = test_dir("empty");
        cleanup(&dir);
        let backups = list_backups(&dir.join("nonexistent")).unwrap();
        assert!(backups.is_empty());
    }
}
