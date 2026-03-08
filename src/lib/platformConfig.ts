import type { PlatformConnection } from './platform';
import {
  startTwitchAuth,
  refreshTwitchTokens,
  revokeTwitchAuth,
  startYouTubeAuth,
  refreshYouTubeTokens,
  revokeYouTubeAuth,
  startKickAuth,
  refreshKickTokens,
  revokeKickAuth,
} from './platform';

export type PlatformId = 'twitch' | 'youtube' | 'kick';

export interface PlatformConfig {
  id: PlatformId;
  label: string;
  brandColor: string;
  startAuth: () => Promise<PlatformConnection>;
  refreshTokens: (connectionId: string) => Promise<void>;
  revokeAuth: (connectionId: string) => Promise<void>;
}

export const PLATFORMS: PlatformConfig[] = [
  {
    id: 'twitch',
    label: 'Twitch',
    brandColor: '#9146FF',
    startAuth: startTwitchAuth,
    refreshTokens: refreshTwitchTokens,
    revokeAuth: revokeTwitchAuth,
  },
  {
    id: 'youtube',
    label: 'YouTube',
    brandColor: '#FF0000',
    startAuth: startYouTubeAuth,
    refreshTokens: refreshYouTubeTokens,
    revokeAuth: revokeYouTubeAuth,
  },
  {
    id: 'kick',
    label: 'Kick',
    brandColor: '#53FC18',
    startAuth: startKickAuth,
    refreshTokens: refreshKickTokens,
    revokeAuth: revokeKickAuth,
  },
];
