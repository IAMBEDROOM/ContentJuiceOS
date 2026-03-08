import { useEffect, useState, useRef, useCallback } from 'react';
import { Routes, Route, NavLink } from 'react-router-dom';
import { Socket } from 'socket.io-client';
import { getSocketIoInfo, connectToNamespace } from './lib/socket';
import ConnectionsPage from './pages/ConnectionsPage';
import './App.css';

interface PongResponse {
  message: string;
  timestamp: string;
  receivedData: unknown;
}

function HomePage() {
  return (
    <div className="home-page">
      <h1>ContentJuiceOS</h1>
      <p className="subtitle">Content Creator Operating System</p>
      <div className="status-pill">
        <span className="status-dot" />
        System Ready
      </div>
    </div>
  );
}

function App() {
  const [socketStatus, setSocketStatus] = useState<string>('disconnected');
  const [pongLog, setPongLog] = useState<PongResponse[]>([]);
  const socketRef = useRef<Socket | null>(null);

  useEffect(() => {
    let socket: Socket | null = null;

    getSocketIoInfo()
      .then((info) => {
        socket = connectToNamespace(info.baseUrl, '/overlays');
        socketRef.current = socket;

        socket.on('connect', () => setSocketStatus('connected'));
        socket.on('disconnect', () => setSocketStatus('disconnected'));
        socket.on('connect_error', () => setSocketStatus('error'));
        socket.on('pong', (data: PongResponse) => {
          setPongLog((prev) => [data, ...prev].slice(0, 10));
        });
      })
      .catch(() => setSocketStatus('error'));

    return () => {
      socket?.disconnect();
      socketRef.current = null;
    };
  }, []);

  const sendPing = useCallback(() => {
    const socket = socketRef.current;
    if (!socket?.connected) return;
    socket.emit('ping', { sentAt: new Date().toISOString() });
  }, []);

  const statusColor =
    socketStatus === 'connected' ? '#22c55e' : socketStatus === 'error' ? '#FF007F' : '#888';

  return (
    <div className="app-layout">
      <nav className="app-nav">
        <NavLink to="/">Home</NavLink>
        <NavLink to="/connections">Connections</NavLink>
        <div className="nav-spacer" />
        <div className="status-pill">
          <span className="status-dot" style={{ backgroundColor: statusColor }} />
          Socket.IO: {socketStatus}
        </div>
      </nav>

      <main className="app-main">
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/connections" element={<ConnectionsPage />} />
        </Routes>
      </main>

      <div className="socketio-section">
        <button className="ping-button" onClick={sendPing} disabled={socketStatus !== 'connected'}>
          Send Ping
        </button>
        {pongLog.length > 0 && (
          <div className="pong-log">
            {pongLog.map((entry, i) => (
              <div key={i} className="pong-entry">
                pong @ {entry.timestamp}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
