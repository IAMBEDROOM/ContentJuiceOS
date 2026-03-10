import { invoke } from '@tauri-apps/api/core';

// ---------------------------------------------------------------------------
// Asset directory management
// ---------------------------------------------------------------------------

/** Returns the absolute path to the resolved asset root directory. */
export async function getAssetRoot(): Promise<string> {
  return invoke<string>('get_asset_root');
}

/** Ensures all asset subdirectories exist, returning the asset root path. */
export async function ensureAssetDirectories(): Promise<string> {
  return invoke<string>('ensure_asset_directories');
}
