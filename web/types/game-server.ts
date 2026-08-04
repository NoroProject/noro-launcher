/** Игровой сервер, зарегистрированный под сборкой. */
export interface GameServer {
  id: string;
  server_id: string;
  name: string;
  mc_host: string;
  mc_port: number;
  sort_order: number;
  online: number;
  max_online: number;
  version: string | null;
  last_seen_at: string | null;
  created_at: string;
  /** Агент выходил на связь недавно. */
  live: boolean;
  /** `proxy` — точка входа, `server` — бэкенд с агентом. */
  kind: GameServerKind;
}

export type GameServerKind = "proxy" | "server";

export interface GameServerForm {
  name: string;
  mc_host: string;
  mc_port: number;
  sort_order: number;
  kind: GameServerKind;
}
