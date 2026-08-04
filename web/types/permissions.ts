/** Подсказка каталога узлов: `GET /api/admin/permission-nodes`. */
export interface PermissionSuggestion {
  /** Сам узел, например `noro.admin.users`. */
  node: string;
  /** `game` — принёс агент игрового сервера, `launcher` — вывел мастер. */
  source: 'game' | 'launcher';
  /** Человекочитаемое пояснение; у игровых узлов его нет. */
  label: string | null;
}

/** Право с контекстом сборки. `server_id: null` — глобально. */
export interface PermissionEntry {
  permission: string;
  server_id: string | null;
}

/**
 * Мастер пока отдаёт только плоский список узлов, без контекста. Когда появится
 * `permission_grants`, берём его; до тех пор считаем все права глобальными.
 */
export function toPermissionEntries(
  grants: PermissionEntry[] | undefined,
  nodes: string[]
): PermissionEntry[] {
  if (grants) return grants
  return nodes.map(permission => ({ permission, server_id: null }))
}

/** Ключ строки списка: одно право может быть выдано в нескольких контекстах. */
export function permissionKey(entry: PermissionEntry) {
  return `${entry.server_id || 'global'}:${entry.permission}`
}
