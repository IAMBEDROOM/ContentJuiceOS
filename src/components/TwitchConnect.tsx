import { useEffect, useState, useCallback } from 'react';
import {
  PlatformConnection,
  getPlatformConnections,
  startTwitchAuth,
  disconnectPlatform,
  refreshTwitchTokens,
} from '../lib/platform';

type AuthPhase = 'idle' | 'authorizing' | 'error';

export default function TwitchConnect() {
  const [connection, setConnection] = useState<PlatformConnection | null>(null);
  const [phase, setPhase] = useState<AuthPhase>('idle');
  const [error, setError] = useState<string | null>(null);

  const loadConnection = useCallback(async () => {
    try {
      const connections = await getPlatformConnections();
      const twitch = connections.find((c) => c.platform === 'twitch') ?? null;
      setConnection(twitch);
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
      const conn = await startTwitchAuth();
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
      await refreshTwitchTokens(connection.id);
      await loadConnection();
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const isConnected = connection?.status === 'connected';
  const isExpired = connection?.status === 'expired';

  return (
    <div className="twitch-connect">
      <div className="twitch-header">
        <span className="twitch-label">Twitch</span>
        <span
          className="status-dot"
          style={{
            backgroundColor: isConnected
              ? '#22c55e'
              : isExpired
                ? '#FFD600'
                : '#888',
          }}
        />
        <span className="twitch-status">
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
        <div className="twitch-profile">
          {connection.avatarUrl && (
            <img
              src={connection.avatarUrl}
              alt={connection.displayName}
              className="twitch-avatar"
            />
          )}
          <span className="twitch-username">{connection.displayName}</span>
        </div>
      )}

      <div className="twitch-actions">
        {!isConnected && phase !== 'authorizing' && (
          <button className="ping-button" onClick={handleConnect}>
            Connect Twitch
          </button>
        )}

        {phase === 'authorizing' && (
          <span className="twitch-waiting">Waiting for authorization...</span>
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

      {error && <div className="twitch-error">{error}</div>}
    </div>
  );
}
