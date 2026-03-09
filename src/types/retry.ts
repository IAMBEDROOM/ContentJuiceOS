export type HealthState = 'healthy' | 'degraded' | 'down';

export interface PlatformHealthStatus {
  platform: string;
  state: HealthState;
  consecutiveFailures: number;
  lastSuccess: string | null;
  lastFailure: string | null;
  lastErrorMessage: string | null;
  queuedActions: number;
}

export interface QueueStats {
  platform: string;
  pendingCount: number;
  oldestAgeSecs: number | null;
}
