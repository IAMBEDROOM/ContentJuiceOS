import { invoke } from '@tauri-apps/api/core';

export interface ServerInfo {
  port: number;
  baseUrl: string;
}

export async function getServerInfo(): Promise<ServerInfo> {
  return invoke('get_server_info');
}
