/** Подсказка каталога узлов: `GET /api/admin/permission-nodes`. */
export interface PermissionSuggestion {
  /** Сам узел, например `noro.admin.users`. */
  node: string;
  /** `game` — принёс агент игрового сервера, `launcher` — вывел мастер. */
  source: 'game' | 'launcher';
  /** Человекочитаемое пояснение; у игровых узлов его нет. */
  label: string | null;
  /** Раздел для группировки в редакторе ролей. */
  group?: string | null;
}

/** Право с контекстом сборки. `server_id: null` — глобально. */
export interface PermissionEntry {
  permission: string;
  server_id: string | null;
}

/**
 * Контекст берём из `permission_grants`. Плоский список узлов остаётся запасным
 * путём: у него контекста нет вовсе, и всё в нём приходится считать глобальным —
 * пока мастер не отдавал грантов, из-за этого точечно выданное право выглядело
 * в админке выданным везде.
 */
export function toPermissionEntries(
  grants: PermissionEntry[] | undefined,
  nodes: string[]
): PermissionEntry[] {
  if (grants) return grants
  return nodes.map(permission => ({ permission, server_id: null }))
}

