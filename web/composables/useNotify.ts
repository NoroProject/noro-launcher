/**
 * Единая обратная связь для действий админки.
 *
 * До этого обработчики писались как `try { ... } finally { busy = null }` — без
 * `catch`. Успех выглядел ровно как отказ: форма уже показывает то, что ввели,
 * и после нажатия на экране не менялось ничего. Упавший запрос при этом уходил
 * в unhandled rejection и не доходил до пользователя вообще.
 */
export function useNotify() {
  const toast = useToast()

  return {
    ok(title = 'Saved', description?: string) {
      toast.add({ title, description, icon: 'i-lucide-check', color: 'success' })
    },

    fail(err: unknown, title = 'Failed') {
      console.error(err)
      toast.add({ title, description: describe(err), icon: 'i-lucide-triangle-alert', color: 'error' })
    },
  }
}

/** Достаёт текст ошибки мастера; `$fetch` прячет тело ответа в `data`. */
function describe(err: unknown): string {
  const e = err as { data?: { error?: string, message?: string }, statusCode?: number, message?: string }
  const fromBody = e?.data?.error || e?.data?.message
  if (fromBody) return fromBody
  if (e?.statusCode === 401 || e?.statusCode === 403) return 'Not enough permissions, or the session expired.'
  if (e?.statusCode) return `Master returned ${e.statusCode}.`
  return e?.message || 'Unknown error.'
}
