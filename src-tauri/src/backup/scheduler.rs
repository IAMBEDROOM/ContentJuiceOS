use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use log::{error, info};
use tauri::{AppHandle, Manager};

use super::engine;
use crate::db::Database;

pub struct BackupScheduler {
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl BackupScheduler {
    pub fn start(app_handle: AppHandle, app_data_dir: PathBuf) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let backup_dir = app_data_dir.join("backups");

        let handle = thread::spawn(move || {
            info!("Backup scheduler started");

            loop {
                if shutdown_flag.load(Ordering::Relaxed) {
                    break;
                }

                // Read settings
                let (interval_hours, max_backups) = {
                    let db = app_handle.state::<Database>();
                    let conn = match db.conn.lock() {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Backup scheduler: failed to lock database: {e}");
                            sleep_with_check(&shutdown_flag, 60);
                            continue;
                        }
                    };

                    let interval: i64 = conn
                        .query_row(
                            "SELECT value FROM settings WHERE key = 'general.backupIntervalHours'",
                            [],
                            |row| row.get(0),
                        )
                        .and_then(|v: String| v.parse::<i64>().map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
                        }))
                        .unwrap_or(24);

                    let max: u32 = conn
                        .query_row(
                            "SELECT value FROM settings WHERE key = 'general.maxBackups'",
                            [],
                            |row| row.get(0),
                        )
                        .and_then(|v: String| v.parse::<u32>().map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
                        }))
                        .unwrap_or(7);

                    (interval, max)
                };

                // Check if backup is due
                let backup_due = match engine::list_backups(&backup_dir) {
                    Ok(backups) => {
                        if backups.is_empty() {
                            true
                        } else {
                            // Parse the newest backup's timestamp from filename
                            let newest = &backups[0].filename;
                            is_backup_overdue(newest, interval_hours)
                        }
                    }
                    Err(_) => true,
                };

                if backup_due {
                    let db = app_handle.state::<Database>();
                    let conn = match db.conn.lock() {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Backup scheduler: failed to lock database for backup: {e}");
                            sleep_with_check(&shutdown_flag, 60);
                            continue;
                        }
                    };

                    match engine::create_backup(&conn, &backup_dir) {
                        Ok(info) => {
                            info!("Automatic backup created: {}", info.filename);
                            if let Err(e) = engine::cleanup_old_backups(&backup_dir, max_backups) {
                                error!("Failed to cleanup old backups: {e}");
                            }
                        }
                        Err(e) => error!("Automatic backup failed: {e}"),
                    }
                }

                // Sleep for 60 seconds, checking shutdown each second
                sleep_with_check(&shutdown_flag, 60);
            }

            info!("Backup scheduler stopped");
        });

        Self {
            shutdown,
            handle: Some(handle),
        }
    }

    pub fn stop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for BackupScheduler {
    fn drop(&mut self) {
        self.stop();
    }
}

fn sleep_with_check(shutdown: &AtomicBool, seconds: u64) {
    for _ in 0..seconds {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn is_backup_overdue(filename: &str, interval_hours: i64) -> bool {
    // Extract timestamp from filename like "contentjuiceos_backup_20260306_143022.db"
    let timestamp_part = filename
        .trim_start_matches("_prerestore_")
        .trim_start_matches("contentjuiceos_backup_")
        .trim_end_matches(".db");

    let parsed = chrono::NaiveDateTime::parse_from_str(timestamp_part, "%Y%m%d_%H%M%S");
    match parsed {
        Ok(backup_time) => {
            let now = chrono::Local::now().naive_local();
            let age = now.signed_duration_since(backup_time);
            age.num_hours() >= interval_hours
        }
        Err(_) => true, // If we can't parse, assume overdue
    }
}
