import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

import type { Asset, AssetListResponse, AssetReference, AssetType, DeleteAssetsResponse } from '../types/platform';

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

// ---------------------------------------------------------------------------
// Asset CRUD
// ---------------------------------------------------------------------------

/** Lists assets with optional filtering, search, and pagination. */
export async function listAssets(params?: {
  typeFilter?: AssetType;
  search?: string;
  limit?: number;
  offset?: number;
}): Promise<AssetListResponse> {
  return invoke<AssetListResponse>('list_assets', {
    typeFilter: params?.typeFilter ?? null,
    search: params?.search ?? null,
    limit: params?.limit ?? null,
    offset: params?.offset ?? null,
  });
}

/** Imports a file from the given source path into the asset library. */
export async function importAsset(sourcePath: string): Promise<Asset> {
  return invoke<Asset>('import_asset', { sourcePath });
}

/** Returns the absolute file path for an asset by ID. */
export async function getAssetFilePath(id: string): Promise<string> {
  return invoke<string>('get_asset_file_path', { id });
}

// ---------------------------------------------------------------------------
// Asset deletion
// ---------------------------------------------------------------------------

/** Checks what other entities reference a given asset. */
export async function checkAssetReferences(id: string): Promise<AssetReference[]> {
  return invoke<AssetReference[]>('check_asset_references', { id });
}

/** Deletes a single asset (DB + file). Set force=true to delete even if referenced. */
export async function deleteAsset(id: string, force = false): Promise<void> {
  return invoke<void>('delete_asset', { id, force });
}

/** Deletes multiple assets. Returns count of deleted and any failures. */
export async function deleteAssetsBatch(ids: string[], force = false): Promise<DeleteAssetsResponse> {
  return invoke<DeleteAssetsResponse>('delete_assets_batch', { ids, force });
}

// ---------------------------------------------------------------------------
// File URL helpers
// ---------------------------------------------------------------------------

/** Converts an absolute file path to an asset:// URL for use in img/audio/video src. */
export function assetFileUrl(absolutePath: string): string {
  return convertFileSrc(absolutePath);
}

// ---------------------------------------------------------------------------
// Native file dialog
// ---------------------------------------------------------------------------

/** Opens a native file picker dialog for selecting assets to import. */
export async function openImportDialog(): Promise<string[] | null> {
  const result = await open({
    multiple: true,
    title: 'Import Assets',
    filters: [
      {
        name: 'Media Files',
        extensions: [
          'png', 'jpg', 'jpeg', 'gif', 'webp', 'svg',
          'mp3', 'wav', 'ogg', 'flac', 'aac',
          'mp4', 'webm', 'mov', 'mkv',
          'ttf', 'otf', 'woff', 'woff2',
          'json', 'srt', 'vtt', 'ass',
        ],
      },
    ],
  });

  if (!result) return null;
  // Normalize: dialog returns string | string[] depending on `multiple`
  return Array.isArray(result) ? result : [result];
}
