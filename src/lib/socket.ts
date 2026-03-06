import { invoke } from '@tauri-apps/api/core';
import { io, Socket } from 'socket.io-client';

export interface SocketIoInfo {
  port: number;
  baseUrl: string;
  namespaces: string[];
}

export async function getSocketIoInfo(): Promise<SocketIoInfo> {
  return invoke('get_socket_io_info');
}

export function connectToNamespace(baseUrl: string, namespace: '/overlays' | '/control'): Socket {
  return io(`${baseUrl}${namespace}`, {
    transports: ['websocket', 'polling'],
    reconnection: true,
    reconnectionAttempts: 5,
    reconnectionDelay: 1000,
  });
}
