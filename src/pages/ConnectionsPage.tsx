import { useEffect, useState, useCallback, useRef } from 'react';
import type { PlatformConnection } from '../lib/platform';
import { getPlatformConnections, disconnectPlatform } from '../lib/platform';
import { PLATFORMS } from '../lib/platformConfig';
import type { PlatformId } from '../lib/platformConfig';
import PlatformCard from '../components/PlatformCard';
import type { AuthPhase } from '../components/PlatformCard';
import './ConnectionsPage.css';

export default function ConnectionsPage() {
  const [connections, setConnections] = useState<Record<string, PlatformConnection | null>>({});
  const [phases, setPhases] = useState<Record<string, AuthPhase>>({});
  const [errors, setErrors] = useState<Record<string, string | null>>({});
  const [loading, setLoading] = useState(true);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const loadConnections = useCallback(async () => {
    try {
      const all = await getPlatformConnections();
      if (!mountedRef.current) return;
      const map: Record<string, PlatformConnection | null> = {};
      for (const p of PLATFORMS) {
        map[p.id] = all.find((c) => c.platform === p.id) ?? null;
      }
      setConnections(map);
    } catch {
      // Will show all platforms as disconnected
    } finally {
      if (mountedRef.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadConnections();
  }, [loadConnections]);

  const handleConnect = async (platformId: PlatformId) => {
    const config = PLATFORMS.find((p) => p.id === platformId);
    if (!config) return;

    setPhases((prev) => ({ ...prev, [platformId]: 'authorizing' }));
    setErrors((prev) => ({ ...prev, [platformId]: null }));

    try {
      const conn = await config.startAuth();
      if (!mountedRef.current) return;
      setConnections((prev) => ({ ...prev, [platformId]: conn }));
      setPhases((prev) => ({ ...prev, [platformId]: 'idle' }));
    } catch (e) {
      if (!mountedRef.current) return;
      setErrors((prev) => ({ ...prev, [platformId]: String(e) }));
      setPhases((prev) => ({ ...prev, [platformId]: 'error' }));
    }
  };

  const handleDisconnect = async (platformId: PlatformId) => {
    const conn = connections[platformId];
    if (!conn) return;

    try {
      await disconnectPlatform(conn.id);
      if (!mountedRef.current) return;
      setConnections((prev) => ({ ...prev, [platformId]: null }));
      setPhases((prev) => ({ ...prev, [platformId]: 'idle' }));
      setErrors((prev) => ({ ...prev, [platformId]: null }));
    } catch (e) {
      if (!mountedRef.current) return;
      setErrors((prev) => ({ ...prev, [platformId]: String(e) }));
    }
  };

  const handleRevoke = async (platformId: PlatformId) => {
    const config = PLATFORMS.find((p) => p.id === platformId);
    const conn = connections[platformId];
    if (!config || !conn) return;

    try {
      await config.revokeAuth(conn.id);
      if (!mountedRef.current) return;
      setConnections((prev) => ({ ...prev, [platformId]: null }));
      setPhases((prev) => ({ ...prev, [platformId]: 'idle' }));
      setErrors((prev) => ({ ...prev, [platformId]: null }));
    } catch (e) {
      if (!mountedRef.current) return;
      setErrors((prev) => ({ ...prev, [platformId]: String(e) }));
    }
  };

  const handleRefresh = async (platformId: PlatformId) => {
    const config = PLATFORMS.find((p) => p.id === platformId);
    const conn = connections[platformId];
    if (!config || !conn) return;

    try {
      await config.refreshTokens(conn.id);
      if (!mountedRef.current) return;
      await loadConnections();
      setErrors((prev) => ({ ...prev, [platformId]: null }));
    } catch (e) {
      if (!mountedRef.current) return;
      setErrors((prev) => ({ ...prev, [platformId]: String(e) }));
    }
  };

  if (loading) {
    return (
      <div className="connections-page">
        <h2 className="connections-title">Platform Connections</h2>
        <p className="connections-loading">Loading connections...</p>
      </div>
    );
  }

  return (
    <div className="connections-page">
      <h2 className="connections-title">Platform Connections</h2>
      <p className="connections-subtitle">
        Connect your streaming platforms to enable chat, alerts, and analytics.
      </p>
      <div className="connections-grid">
        {PLATFORMS.map((config) => (
          <PlatformCard
            key={config.id}
            config={config}
            connection={connections[config.id] ?? null}
            phase={phases[config.id] ?? 'idle'}
            error={errors[config.id] ?? null}
            onConnect={() => handleConnect(config.id)}
            onDisconnect={() => handleDisconnect(config.id)}
            onRevoke={() => handleRevoke(config.id)}
            onRefresh={() => handleRefresh(config.id)}
          />
        ))}
      </div>
    </div>
  );
}
