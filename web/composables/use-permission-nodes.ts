import type { Ref } from 'vue'
import type { PermissionSuggestion } from '~/types/permissions'

/**
 * Каталог узлов для автодополнения. Лаунчерные права приходят всегда, игровые —
 * только вместе с контекстом сборки, поэтому каталог перезапрашивается при смене
 * контекста и кэшируется на время жизни экрана.
 */
export function usePermissionNodes(serverId: Ref<string>) {
  const auth = useAuth()
  const suggestions = ref<PermissionSuggestion[]>([])
  const pending = ref(false)
  const error = ref<string | null>(null)
  const cache = new Map<string, PermissionSuggestion[]>()

  async function load(id: string) {
    const cached = cache.get(id)
    if (cached) {
      suggestions.value = cached
      return
    }
    pending.value = true
    error.value = null
    try {
      const query = id ? `?server_id=${encodeURIComponent(id)}` : ''
      const list = await auth.request<PermissionSuggestion[]>(`/api/admin/permission-nodes${query}`)
      cache.set(id, list)
      suggestions.value = list
    } catch (e) {
      // Каталог — подсказка, а не источник правды: без него право всё равно можно ввести руками.
      error.value = humanError(e)
      suggestions.value = []
    } finally {
      pending.value = false
    }
  }

  // Каталог живёт только в интерактиве — на SSR его тянуть незачем.
  if (import.meta.client) {
    watch(serverId, async id => await load(id), { immediate: true })
  }

  return { suggestions, pending, error }
}
