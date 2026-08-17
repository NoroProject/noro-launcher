/** Витрина проекта: то, что видно на сайте без входа. */

export interface PublicServer {
  id: string;
  name: string;
  description: string;
  icon_url?: string | null;
  background_url?: string | null;
  modloader: string;
  mc_version: string;
  /** Адрес входа для игрока. Пусто — адрес ещё не заведён. */
  address: string;
  online: number;
  max_online: number;
  /** Сервер отвечает: без этого онлайн — последнее, что он сказал перед падением. */
  live: boolean;
}
