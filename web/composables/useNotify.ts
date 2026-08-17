/**
 * Единая обратная связь для действий админки.
 */
export function useNotify() {
  const toast = useToast()
  const { t } = useT()

  return {
    ok(title?: string, description?: string) {
      toast.add({
        title: title || t('toast-success'),
        description,
        icon: 'i-lucide-check',
        color: 'success'
      })
    },

    /** Нейтральное уведомление о начатом действии: скачивание, копирование. */
    info(title: string, description?: string) {
      toast.add({
        title,
        description,
        icon: 'i-lucide-info',
        color: 'info'
      })
    },

    fail(err: unknown, title?: string) {
      console.error(err)
      toast.add({
        title: title || t('toast-error'),
        description: describe(err, t),
        icon: 'i-lucide-triangle-alert',
        color: 'error'
      })
    },
  }
}

/** Достаёт текст ошибки мастера; `$fetch` прячет тело ответа в `data`. */
function describe(err: unknown, t: (key: string, args?: Record<string, string | number>) => string): string {
  const e = err as { data?: { error?: string, message?: string }, statusCode?: number, message?: string }
  const fromBody = e?.data?.error || e?.data?.message
  if (fromBody) return fromBody
  if (e?.statusCode === 401 || e?.statusCode === 403) return t('auth-session-expired')
  if (e?.statusCode) return t('notif-server-error', { reason: String(e.statusCode) })
  return e?.message || t('toast-error')
}
