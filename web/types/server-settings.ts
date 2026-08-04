import type { BuildRow, Role } from "./api";

export type ServerSettingsTab = "server" | "client";

/**
 * Формат импортируемого модпака. Имя совпадает с сегментом маршрута
 * `/api/admin/builds/{id}/import/{kind}`.
 *
 * `zip` — обычный архив, внутри которого лежит корень сборки: `mods/`,
 * `config/` и прочее. Манифеста в нём нет, ничего не докачивается.
 */
export type PackImportKind = "mrpack" | "curseforge" | "zip";
export type ServerAssetKind = "icon" | "background";

export interface ServerEditForm {
  name: string;
  description: string;
  modloader: string;
  mc_version: string;
  active: boolean;
  limited: boolean;
  sort_order: number;
}

export interface BuildCreateForm {
  server_id: string;
  version: string;
  modloader: string;
  modloader_version: string;
  mc_version: string;
}

export interface ServerEditorState {
  builds: BuildRow[];
  roles: Role[];
}

/** Прогресс импорта модпака. Зеркалит `ImportProgress` на мастере. */
export interface ImportProgress {
  total: number;
  current: number;
  current_file: string;
  done: boolean;
  error?: string | null;
  warnings: string[];
  recommended_mc_version?: string | null;
  recommended_modloader_version?: string | null;
}
