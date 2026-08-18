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
  icon_url?: string | null;
  last_seen_at: string | null;
  created_at: string;
  live: boolean;
  kind: GameServerKind;
  maintenance: boolean;
  maintenance_reason: string | null;
}

export type GameServerKind = "proxy" | "server";

export interface GameServerForm {
  name: string;
  mc_host: string;
  mc_port: number;
  sort_order: number;
  kind: GameServerKind;
  maintenance?: boolean;
  maintenance_reason?: string;
  /**
   * Обязательное: поле привязано к `v-model.number`, а тот не принимает
   * `undefined`. Значение по умолчанию задаёт форма, а не тип.
   */
  countdown_seconds: number;
}
