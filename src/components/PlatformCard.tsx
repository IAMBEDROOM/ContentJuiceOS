import type { PlatformConfig } from '../lib/platformConfig';
import type { PlatformConnection } from '../lib/platform';

export type AuthPhase = 'idle' | 'authorizing' | 'error';

interface PlatformCardProps {
  config: PlatformConfig;
  connection: PlatformConnection | null;
  phase: AuthPhase;
  error: string | null;
  onConnect: () => void;
  onDisconnect: () => void;
  onRevoke: () => void;
  onRefresh: () => void;
}

function statusInfo(connection: PlatformConnection | null) {
  if (!connection) return { color: '#555', label: 'Not Connected' };
  switch (connection.status) {
    case 'connected':
      return { color: '#22c55e', label: 'Connected' };
    case 'expired':
      return { color: '#FFD600', label: 'Expired' };
    case 'revoked':
      return { color: '#FF007F', label: 'Revoked' };
    default:
      return { color: '#555', label: 'Not Connected' };
  }
}

function parseScopes(scopes: string): string[] {
  try {
    const parsed: unknown = JSON.parse(scopes);
    if (Array.isArray(parsed)) return parsed.map(String);
  } catch {
    // Not JSON — treat as plain string
  }
  return scopes ? [scopes] : [];
}

function formatDate(dateStr: string | null): string | null {
  if (!dateStr) return null;
  return new Date(dateStr).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

export default function PlatformCard({
  config,
  connection,
  phase,
  error,
  onConnect,
  onDisconnect,
  onRevoke,
  onRefresh,
}: PlatformCardProps) {
  const status = statusInfo(connection);
  const isConnected = connection?.status === 'connected';
  const isExpired = connection?.status === 'expired';
  const isRevoked = connection?.status === 'revoked';
  const showConnect =
    (!connection || connection.status === 'disconnected' || isRevoked) && phase !== 'authorizing';

  return (
    <div className="platform-card" style={{ borderLeftColor: config.brandColor }}>
      <div className="platform-card-header">
        <div className="platform-card-title">
          <span className="status-dot" style={{ backgroundColor: config.brandColor }} />
          <span className="platform-card-label">{config.label}</span>
        </div>
        <span
          className="platform-status-pill"
          style={{ backgroundColor: `${status.color}22`, color: status.color }}
        >
          <span className="status-dot" style={{ backgroundColor: status.color }} />
          {status.label}
        </span>
      </div>

      {(isConnected || isExpired) && connection && (
        <div className="platform-card-profile">
          {connection.avatarUrl && (
            <img
              src={connection.avatarUrl}
              alt={connection.displayName}
              className="platform-avatar"
            />
          )}
          <div className="platform-card-info">
            <span className="platform-display-name">{connection.displayName}</span>
            <span className="platform-username">@{connection.platformUsername}</span>
          </div>
        </div>
      )}

      {(isConnected || isExpired) && connection && (
        <div className="platform-card-details">
          {parseScopes(connection.scopes).length > 0 && (
            <div className="platform-detail">
              <span className="platform-detail-label">Scopes</span>
              <span className="platform-detail-value">
                {parseScopes(connection.scopes).join(', ')}
              </span>
            </div>
          )}
          {connection.connectedAt && (
            <div className="platform-detail">
              <span className="platform-detail-label">Connected</span>
              <span className="platform-detail-value">{formatDate(connection.connectedAt)}</span>
            </div>
          )}
          {connection.lastRefreshedAt && (
            <div className="platform-detail">
              <span className="platform-detail-label">Last Refreshed</span>
              <span className="platform-detail-value">
                {formatDate(connection.lastRefreshedAt)}
              </span>
            </div>
          )}
        </div>
      )}

      <div className="platform-card-actions">
        {showConnect && (
          <button className="btn btn-connect" onClick={onConnect}>
            Connect
          </button>
        )}

        {phase === 'authorizing' && <span className="platform-authorizing">Authorizing...</span>}

        {isExpired && (
          <button className="btn btn-refresh" onClick={onRefresh}>
            Refresh Token
          </button>
        )}

        {isConnected && (
          <button className="btn btn-disconnect" onClick={onDisconnect}>
            Disconnect
          </button>
        )}

        {(isConnected || isExpired) && (
          <button className="btn btn-revoke" onClick={onRevoke}>
            Revoke
          </button>
        )}
      </div>

      {error && <div className="platform-card-error">{error}</div>}
    </div>
  );
}
