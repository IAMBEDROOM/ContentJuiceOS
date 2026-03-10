import { invoke } from '@tauri-apps/api/core';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type CredentialKind =
  | { type: 'platformToken'; connectionId: string }
  | { type: 'apiKey'; service: string }
  | { type: 'appSecret'; key: string };

export interface OAuthTokens {
  accessToken: string;
  refreshToken?: string;
  tokenExpiresAt?: string;
}

export type CredentialBackend = 'keychain' | 'encrypted_sqlite';

// ---------------------------------------------------------------------------
// Raw credential operations
// ---------------------------------------------------------------------------

export async function storeCredential(kind: CredentialKind, value: string): Promise<void> {
  await invoke('store_credential', { kind, value });
}

export async function getCredential(kind: CredentialKind): Promise<string | null> {
  return invoke<string | null>('get_credential', { kind });
}

export async function deleteCredential(kind: CredentialKind): Promise<void> {
  await invoke('delete_credential', { kind });
}

export async function hasCredential(kind: CredentialKind): Promise<boolean> {
  return invoke<boolean>('has_credential', { kind });
}

export async function getCredentialBackend(): Promise<CredentialBackend> {
  return invoke<CredentialBackend>('get_credential_backend');
}

// ---------------------------------------------------------------------------
// OAuth token convenience methods
// ---------------------------------------------------------------------------

export async function storePlatformTokens(
  connectionId: string,
  tokens: OAuthTokens,
): Promise<void> {
  await invoke('store_platform_tokens', {
    connectionId,
    accessToken: tokens.accessToken,
    refreshToken: tokens.refreshToken ?? null,
    expiresAt: tokens.tokenExpiresAt ?? null,
  });
}

export async function getPlatformTokens(connectionId: string): Promise<OAuthTokens | null> {
  return invoke<OAuthTokens | null>('get_platform_tokens', { connectionId });
}
