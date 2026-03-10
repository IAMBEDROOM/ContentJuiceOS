import { invoke } from '@tauri-apps/api/core';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface PlatformConnection {
  id: string;
  platform: string;
  platformUserId: string;
  platformUsername: string;
  displayName: string;
  avatarUrl: string | null;
  scopes: string;
  status: 'connected' | 'disconnected' | 'expired' | 'revoked';
  connectedAt: string | null;
  lastRefreshedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

// ---------------------------------------------------------------------------
// Platform connection CRUD
// ---------------------------------------------------------------------------

export async function getPlatformConnections(): Promise<PlatformConnection[]> {
  return invoke<PlatformConnection[]>('get_platform_connections');
}

export async function getPlatformConnection(id: string): Promise<PlatformConnection | null> {
  return invoke<PlatformConnection | null>('get_platform_connection', { id });
}

export async function disconnectPlatform(id: string): Promise<void> {
  await invoke('disconnect_platform', { id });
}

// ---------------------------------------------------------------------------
// Twitch-specific
// ---------------------------------------------------------------------------

export async function startTwitchAuth(): Promise<PlatformConnection> {
  return invoke<PlatformConnection>('start_twitch_auth');
}

export async function refreshTwitchTokens(connectionId: string): Promise<void> {
  await invoke('refresh_twitch_tokens', { connectionId });
}

export async function revokeTwitchAuth(connectionId: string): Promise<void> {
  await invoke('revoke_twitch_auth', { connectionId });
}

// ---------------------------------------------------------------------------
// YouTube-specific
// ---------------------------------------------------------------------------

export async function startYouTubeAuth(): Promise<PlatformConnection> {
  return invoke<PlatformConnection>('start_youtube_auth');
}

export async function refreshYouTubeTokens(connectionId: string): Promise<void> {
  await invoke('refresh_youtube_tokens', { connectionId });
}

export async function revokeYouTubeAuth(connectionId: string): Promise<void> {
  await invoke('revoke_youtube_auth', { connectionId });
}

// ---------------------------------------------------------------------------
// Kick-specific
// ---------------------------------------------------------------------------

export async function startKickAuth(): Promise<PlatformConnection> {
  return invoke<PlatformConnection>('start_kick_auth');
}

export async function refreshKickTokens(connectionId: string): Promise<void> {
  await invoke('refresh_kick_tokens', { connectionId });
}

export async function revokeKickAuth(connectionId: string): Promise<void> {
  await invoke('revoke_kick_auth', { connectionId });
}
