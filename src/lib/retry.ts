import { invoke } from '@tauri-apps/api/core';
import type { PlatformHealthStatus, QueueStats } from '../types/retry';

export async function getPlatformHealth(
  platform: string
): Promise<PlatformHealthStatus> {
  return invoke('get_platform_health', { platform });
}

export async function getAllPlatformHealth(): Promise<PlatformHealthStatus[]> {
  return invoke('get_all_platform_health');
}

export async function getActionQueueStats(
  platform: string
): Promise<QueueStats> {
  return invoke('get_action_queue_stats', { platform });
}

export async function drainActionQueue(platform: string): Promise<number> {
  return invoke('drain_action_queue', { platform });
}
