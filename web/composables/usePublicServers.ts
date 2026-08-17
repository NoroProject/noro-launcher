/** Список серверов проекта с онлайном. Открыт всем, вход не нужен. */

import type { PublicServer } from '~/types/public'

export function usePublicServers() {
  const api = useApi()

  const { data, pending, refresh } = useAsyncData(
    'public-servers',
    async () => {
      try {
        return await api.request<PublicServer[]>('/api/servers')
      } catch {
        // Мастер молчит — страница остаётся рабочей и просто не покажет список.
        return [] as PublicServer[]
      }
    },
    { default: () => [] as PublicServer[] },
  )

  const servers = computed(() => data.value || [])
  const online = computed(() => servers.value.reduce((sum, s) => sum + s.online, 0))
  const anyLive = computed(() => servers.value.some(s => s.live))

  // Онлайн живёт своей жизнью: страницу держат открытой, и цифра пятиминутной
  // давности выглядит как «на сервере никого».
  onMounted(() => {
    const timer = setInterval(refresh, 60_000)
    onBeforeUnmount(() => clearInterval(timer))
  })

  return { servers, online, anyLive, pending, refresh }
}
