import type { PermissionEntry } from '~/types/permissions'

export interface Role {
  id: string;
  name: string;
  display_name: string;
  color?: string | null;
  permissions: string[];
  /** Права с контекстом сборки. */
  permission_grants?: PermissionEntry[];
  is_default: boolean;
  sort_order?: number;
  /** Имя группы LuckPerms, с которой связана роль. */
  lp_group?: string | null;
  /** Юникод-символ рядом с названием роли. Не префикс: это один глиф. */
  icon?: string | null;
  /** Строка перед ником в игре, цвета через `&`. */
  prefix?: string | null;
  /** Строка после ника в игре. */
  suffix?: string | null;
  /** Роль, чьи права действуют и здесь. */
  parent_id?: string | null;
  /** Права, пришедшие от родителя и его родителей. Только для чтения. */
  inherited_permissions?: string[];
}

/** Привязка аккаунта к внешней платформе. */
export interface UserIdentity {
  provider: string;
  provider_user_id: string;
  username?: string | null;
  avatar_url?: string | null;
  /** Платформа регистрации: из неё выведен MC-UUID, отвязать нельзя. */
  is_primary: boolean;
  linked_at: string;
}

export interface UserProfile {
  id: string;
  uuid: string;
  username: string;
  /** Привязанные платформы. Пусто у локального аккаунта. */
  identities: UserIdentity[];
  skin_url?: string | null;
  /** Тонкая модель (Алекс). `false` — классическая (Стив). */
  skin_slim?: boolean;
  cape_url?: string | null;
  roles: Role[];
  permissions: string[];
  /** Прямые права с контекстом сборки. */
  permission_grants?: PermissionEntry[];
  banned?: boolean;
  ban_reason?: string | null;
  created_at?: string | null;
  last_login_at?: string | null;
  frozen?: boolean;
  hide_from_online?: boolean;
  /** Заведён оператором, без входа через платформу. */
  is_local_account?: boolean;
  /** Может заходить в игру. У операторского аккаунта обычно нет. */
  can_play?: boolean;
  /** Единственный аккаунт, который нельзя забанить и удалить. */
  is_root?: boolean;
  /** Заходит на сервер без сообщения в чат. */
  silent_join?: boolean;
}

export interface UserRow {
  id: string;
  identities?: UserIdentity[];
  mc_uuid: string;
  mc_username: string;
  skin_url?: string | null;
  skin_slim?: boolean;
  cape_url?: string | null;
  banned: boolean;
  frozen?: boolean;
  hide_from_online?: boolean;
  ban_reason?: string | null;
  created_at: string;
  last_login_at?: string | null;
}

export interface ServerRow {
  id: string;
  name: string;
  description: string;
  icon_url?: string | null;
  background_url?: string | null;
  modloader: string;
  mc_version: string;
  active: boolean;
  limited: boolean;
  sort_order: number;
  created_at: string;
}

export interface BuildRow {
  id: string;
  server_id: string;
  version: string;
  changelog: string;
  modloader: string;
  modloader_version?: string | null;
  mc_version: string;
  main_class: string;
  jvm_args: unknown;
  game_args: unknown;
  assets_index_name: string;
  published: boolean;
  optional_mods: unknown[];
  allow_optional_mod_suggestions?: boolean;
  recommended_memory_min_mb: number;
  recommended_memory_max_mb: number;
  recommended_jvm_flags: string;
  recommended_show_console_on_launch: boolean;
  unmanaged_paths: string[];
  user_managed_paths: string[];
  manifest_signature?: number[] | null;
  created_at: string;
}

export interface BuildFileRow {
  id: string;
  build_id: string;
  path: string;
  sha1: string;
  size: number;
  side: string;
  kind: string;
}

export interface OptionalMod {
  name: string;
  description: string;
  category: string;
  files: string[];
  enabled_by_default: boolean;
  visible: boolean;
  limited: boolean;
  dependencies: string[];
  conflicts: string[];
  triggers: unknown[];
  icon_url?: string | null;
  author?: string | null;
}

export interface NewsRow {
  id: string;
  title: string;
  body: string;
  preview_img_url?: string | null;
  author_id?: string | null;
  pinned: boolean;
  published_at: string;
}

export interface LauncherVersionRow {
  id: string;
  version: string;
  platform: string;
  sha256: string;
  file_sha1: string;
  size: number;
  signature: string;
  is_current: boolean;
  built_at: string;
  /** `bootstrapper` — то, что качает игрок; `core` — то, что качает bootstrapper. */
  kind: string;
}

export interface ServerCoreRow {
  id: string;
  server_id: string;
  version: string;
  sha256: string;
  file_sha1: string;
  size: number;
  active: boolean;
  uploaded_at: string;
  url: string;
}

export interface AdminTokenRow {
  id: string;
  name: string;
  permissions: string[];
  created_at: string;
  last_used_at?: string | null;
  /** Токен ещё на старой схеме хеширования — перейдёт при первом использовании. */
  legacy_hash: boolean;
}

export interface ModrinthHit {
  project_id: string;
  slug: string;
  title: string;
  description: string;
  icon_url?: string | null;
  author?: string | null;
  downloads: number;
  categories: string[];
}

export interface ModrinthVersion {
  id: string;
  name: string;
  version_number: string;
  version_type: "release" | "beta" | "alpha";
  date_published: string;
}

export interface OptionalAddConfig {
  name: string;
  description: string;
  category: string;
  enabled_by_default: boolean;
  visible: boolean;
  limited: boolean;
  icon_url?: string | null;
  author?: string | null;
}

export interface BuildModSearchResult {
  hits: ModrinthHit[];
  total_hits: number;
}

export interface ImportProgress {
  total: number;
  current: number;
  current_file: string;
  done: boolean;
  error?: string | null;
  warnings: string[];
  recommended_mc_version: string | null;
  recommended_modloader_version: string | null;
}
