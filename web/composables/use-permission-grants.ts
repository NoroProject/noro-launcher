import type { Ref } from 'vue'
import type { PermissionEntry } from '~/types/permissions'

/**
 * Запись прав с контекстом сборки. У ролей и пользователей ручки одинаковые и
 * отличаются только префиксом, поэтому контракт живёт в одном месте: когда мастер
 * научится принимать `server_id`, меняется только этот файл.
 */
export function usePermissionGrants(base: Ref<string>) {
  const auth = useAuth()

  function entryUrl(entry: PermissionEntry) {
    const context = entry.server_id ? `?server_id=${encodeURIComponent(entry.server_id)}` : ''
    return `${base.value}/permissions/${encodeURIComponent(entry.permission)}${context}`
  }

  async function add(entry: PermissionEntry) {
    await auth.request(`${base.value}/permissions`, { method: 'POST', body: entry })
  }

  /** Одно право на несколько сборок — это просто несколько выдач подряд. */
  async function addMany(entries: PermissionEntry[]) {
    for (const entry of entries) {
      await add(entry)
    }
  }

  async function remove(entry: PermissionEntry) {
    await auth.request(entryUrl(entry), { method: 'DELETE' })
  }

  /** Снять право сразу во всех контекстах — крестик в списке делает именно это. */
  async function removeMany(entries: PermissionEntry[]) {
    for (const entry of entries) {
      await remove(entry)
    }
  }

  return { add, addMany, remove, removeMany }
}
