/** Управление игровым сервером через ServerWrapper. */

export interface WrapperInfo {
  platform: string;
  mc_version: string;
  wrapper_version: string;
  server_dir: string;
}

export interface WrapperStatus {
  running: boolean;
  /** Сервер отпечатал `Done (` — принимает игроков. */
  ready: boolean;
  uptime_secs: number;
  exit_code: number | null;
}

export interface WrapperState {
  connected: boolean;
  info: WrapperInfo | null;
  status: WrapperStatus;
}

export type PowerAction = "start" | "stop" | "restart" | "kill";

export interface ServerEntry {
  name: string;
  path: string;
  dir: boolean;
  size: number;
  /** Unix-время в миллисекундах. */
  modified: number;
}

export interface ServerListing {
  path: string;
  entries: ServerEntry[];
}

/** Результат операции на одном сервере при массовом применении. */
export interface ServerOutcome {
  id: string;
  ok: boolean;
  error: string | null;
}

export interface ServerBackup {
  name: string;
  size: number;
  /** Unix-время в миллисекундах. */
  created: number;
}

export interface ServerBackupsResponse {
  backups: ServerBackup[];
}
