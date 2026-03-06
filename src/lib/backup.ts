import { invoke } from "@tauri-apps/api/core";

export interface BackupInfo {
  filename: string;
  createdAt: string;
  sizeBytes: number;
}

export async function createBackup(): Promise<BackupInfo> {
  return invoke("create_backup");
}

export async function listBackups(): Promise<BackupInfo[]> {
  return invoke("list_backups");
}

export async function restoreBackup(filename: string): Promise<void> {
  return invoke("restore_backup", { filename });
}

export async function deleteBackup(filename: string): Promise<void> {
  return invoke("delete_backup", { filename });
}
