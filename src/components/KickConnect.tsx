import { useEffect, useState, useCallback } from 'react';
import {
  PlatformConnection,
  getPlatformConnections,
  startKickAuth,
  disconnectPlatform,
  refreshKickTokens,
} from '../lib/platform';

type AuthPhase = 'idle' | 'authorizing' | 'error';

export default function KickConnect() {
  const [connection, setConnection] = useState<PlatformConnection | null>(null);
  const [phase, setPhase] = useState<AuthPhase>('idle');
  const [error, setError] = useState<string | null>(null);

  const loadConnection = useCallback(async () => {
    try {
      const connections = await getPlatformConnections();
      const kick = connections.find((c) => c.platform === 'kick') ?? null;
      setConnection(kick);
    } catch {
      // Silently handle — component will show disconnected state
    }
  }, []);

  useEffect(() => {
    loadConnection();
  }, [loadConnection]);

  const handleConnect = async () => {
    setPhase('authorizing');
    setError(null);
    try {
      const conn = await startKickAuth();
      setConnection(conn);
      setPhase('idle');
    } catch (e) {
      setError(String(e));
      setPhase('error');
    }
  };

  const handleDisconnect = async () => {
    if (!connection) return;
    try {
      await disconnectPlatform(connection.id);
      setConnection(null);
      setPhase('idle');
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleRefresh = async () => {
    if (!connection) return;
    try {
      await refreshKickTokens(connection.id);
      await loadConnection();
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const isConnected = connection?.status === 'connected';
  const isExpired = connection?.status === 'expired';

  return (
    <div className="kick-connect">
      <div className="kick-header">
        <span className="kick-label">Kick</span>
        <span
          className="status-dot"
          style={{
            backgroundColor: isConnected
              ? '#53FC18'
              : isExpired
                ? '#FFD600'
                : '#888',
          }}
        />
        <span className="kick-status">
          {isConnected
            ? 'Connected'
            : isExpired
              ? 'Expired'
              : connection?.status === 'revoked'
                ? 'Revoked'
                : 'Disconnected'}
        </span>
      </div>

      {isConnected && connection && (
        <div className="kick-profile">
          {connection.avatarUrl && (
            <img
              src={connection.avatarUrl}
              alt={connection.displayName}
              className="kick-avatar"
            />
          )}
          <span className="kick-username">{connection.displayName}</span>
        </div>
      )}

      <div className="kick-actions">
        {!isConnected && phase !== 'authorizing' && (
          <button className="ping-button" onClick={handleConnect}>
            Connect Kick
          </button>
        )}

        {phase === 'authorizing' && (
          <span className="kick-waiting">Waiting for authorization...</span>
        )}

        {isExpired && (
          <button className="ping-button" onClick={handleRefresh}>
            Refresh Token
          </button>
        )}

        {(isConnected || isExpired) && (
          <button
            className="ping-button"
            style={{ backgroundColor: '#FF007F' }}
            onClick={handleDisconnect}
          >
            Disconnect
          </button>
        )}
      </div>

      {error && <div className="kick-error">{error}</div>}
    </div>
  );
}
