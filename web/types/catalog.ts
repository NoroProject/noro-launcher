/** Каталог модов: Modrinth и CurseForge, приведённые мастером к общему виду. */

export type CatalogProvider = "modrinth" | "curseforge";

export interface ModHit {
  provider: CatalogProvider;
  project_id: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  icon_url: string | null;
  downloads: number;
  follows: number;
  categories: string[];
  /** `required` | `optional` | `unsupported` | `unknown`. */
  client_side: string;
  server_side: string;
  updated: string | null;
  page_url: string | null;
}

export interface ModProject extends ModHit {
  /** Markdown у Modrinth, HTML у CurseForge. */
  body: string;
  gallery: string[];
  source_url: string | null;
  issues_url: string | null;
  wiki_url: string | null;
  license: string | null;
  game_versions: string[];
  loaders: string[];
}

export interface ModDependency {
  project_id: string | null;
  kind: string;
}

export interface ModVersion {
  provider: CatalogProvider;
  id: string;
  project_id: string;
  name: string;
  version_number: string;
  game_versions: string[];
  loaders: string[];
  channel: string;
  downloads: number;
  published: string | null;
  filename: string;
  size: number;
  dependencies: ModDependency[];
  downloadable: boolean;
}

export interface CatalogCategory {
  name: string;
  display: string;
  icon: string | null;
}

export interface CatalogPage {
  hits: ModHit[];
  total: number;
  failed: string[];
}

export interface CatalogProviders {
  modrinth: boolean;
  curseforge: boolean;
}

export type ModSource =
  | { kind: "modrinth"; version_id: string }
  | { kind: "curseforge"; project_id: number; file_id: number }
  | { kind: "url"; url: string; filename?: string };

export interface OptionalDraft {
  name: string;
  description: string;
  category: string;
  enabled_by_default: boolean;
  visible: boolean;
  limited: boolean;
  icon_url: string | null;
  author: string | null;
}

export type InstallTarget =
  | { kind: "build"; build_id: string; optional?: OptionalDraft | null }
  | { kind: "game_server"; id: string }
  | { kind: "all_servers"; server_id: string };

export interface InstallResult {
  kind: string;
  id: string;
  label: string;
  ok: boolean;
  path: string | null;
  error: string | null;
}

export interface InstalledModInfo {
  file_id: string;
  path: string;
  sha1: string;
  size: number;
  mod_id?: string | null;
  name?: string | null;
  version?: string | null;
}
