import type { PermissionEntry } from '~/types/permissions'

export interface Role {
  id: string;
  name: string;
  display_name: string;
  color?: string | null;
  permissions: string[];
  /** Права с контекстом сборки. Мастер пока не отдаёт — см. toPermissionEntries. */
  permission_grants?: PermissionEntry[];
  is_default: boolean;
  sort_order?: number;
  /** Имя группы LuckPerms, с которой связана роль. */
  lp_group?: string | null;
  /** Юникод-символ рядом с названием роли. */
  icon?: string | null;
}

export interface UserProfile {
  id: string;
  uuid: string;
  username: string;
  discord_id: string;
  discord_username: string;
  discord_avatar?: string | null;
  skin_url?: string | null;
  cape_url?: string | null;
  roles: Role[];
  permissions: string[];
  /** Прямые права с контекстом сборки. Мастер пока не отдаёт. */
  permission_grants?: PermissionEntry[];
  banned?: boolean;
}

export interface UserRow {
  id: string;
  discord_id: string;
  discord_username: string;
  discord_avatar?: string | null;
  mc_uuid: string;
  mc_username: string;
  skin_url?: string | null;
  cape_url?: string | null;
  banned: boolean;
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
  token_hash: string;
  permissions: string[];
  created_at: string;
  last_used_at?: string | null;
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
