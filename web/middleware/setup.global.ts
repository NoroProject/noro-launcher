/**
 * Увести на визард, пока инстанс не настроен.
 *
 * Без этого ненастроенный сайт выглядит просто сломанным: мастер отвечает
 * 503 на всё, а страницы показывают пустоту без единого намёка, что делать.
 *
 * Проверка идёт один раз за загрузку страницы: состояние установки меняется
 * ровно однажды за жизнь инстанса, и опрашивать мастер на каждый переход
 * незачем.
 */
let checked = false

export default defineNuxtRouteMiddleware(async (to) => {
  if (to.path === '/setup' || checked) return
  // На сервере рендер идёт без браузерной сессии, а сам редирект нужен
  // человеку — проверяем на клиенте.
  if (!import.meta.client) return

  const api = useApi()
  try {
    const status = await api.request<{ setup_completed: boolean }>('/api/setup/status')
    checked = true
    if (!status.setup_completed) return navigateTo('/setup')
  } catch {
    // Мастер не отвечает вовсе — это не повод показывать визард: настраивать
    // нечего, чинить надо мастер. Обычные страницы сами покажут свою ошибку.
    checked = true
  }
})
