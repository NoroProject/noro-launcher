/**
 * Пуш с мастера в открытую вкладку админки.
 *
 * Один сокет на вкладку, а не на страницу: подписчиков может быть несколько,
 * а соединение с мастером стоит одно. Отваливается — переподключаемся с
 * нарастающей паузой и сообщаем об этом подписчикам: им может понадобиться
 * перечитать то, что они пропустили, пока связи не было.
 */

type AdminFrame =
  | { t: 'AuthOk' }
  | { t: 'AuthFail' }
  | { t: 'CaseUpdated', d: { case_id: string } }
  | { t: 'Pong' }

type Handler = (frame: AdminFrame) => void

const handlers = new Set<Handler>()
let socket: WebSocket | null = null
let backoff = 1000
let retry: ReturnType<typeof setTimeout> | null = null
/** Мастер отказал в токене: молчим до следующей подписки с новым. */
let stopped = false

function open(url: string, token: string) {
  if (socket && (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING)) {
    return
  }
  socket = new WebSocket(url)

  socket.onopen = () => {
    backoff = 1000
    socket?.send(JSON.stringify({ t: 'Authenticate', d: { access_token: token } }))
  }

  socket.onmessage = (event) => {
    let frame: AdminFrame
    try {
      frame = JSON.parse(event.data)
    } catch {
      // Незнакомый кадр не повод рвать соединение: мастер обновляется
      // отдельно от сайта и может начать слать то, чего эта вкладка не знает.
      return
    }
    // Токен негоден — переподключаться бессмысленно: следующая попытка пойдёт
    // с ним же. Сессию всё равно перевыдаст обычный запрос, когда получит 401.
    if (frame.t === 'AuthFail') {
      stopped = true
      return
    }
    handlers.forEach(h => h(frame))
  }

  const reconnect = () => {
    socket = null
    if (retry || stopped) return
    retry = setTimeout(() => {
      retry = null
      open(url, token)
    }, backoff)
    backoff = Math.min(backoff * 2, 30_000)
  }
  socket.onclose = reconnect
  socket.onerror = reconnect
}

export function useAdminSocket() {
  const api = useApi()

  /** Подписаться. Возвращает отписку — её зовёт `onUnmounted` страницы. */
  function subscribe(handler: Handler) {
    if (!import.meta.client) return () => {}
    const token = api.token.value
    if (!token) return () => {}

    handlers.add(handler)
    stopped = false
    open(`${api.masterUrl.value.replace(/^http/, 'ws')}/ws/admin`, token)
    return () => {
      handlers.delete(handler)
      // Последний подписчик ушёл — закрываем: держать сокет ради пустой
      // страницы значит держать соединение на каждой открытой вкладке.
      if (handlers.size === 0) {
        socket?.close()
        socket = null
        if (retry) {
          clearTimeout(retry)
          retry = null
        }
      }
    }
  }

  return { subscribe }
}
