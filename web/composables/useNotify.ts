/**
 * Единая обратная связь для действий админки.
 */
export function useNotify() {
  const toast = useToast()
  const { t } = useT()

  /**
   * Показать тост, не плодя одинаковых.
   *
   * Повтор того же сообщения заменяет предыдущее и заводит отсчёт заново:
   * три «Готово» подряд после трёх сохранений ничего не добавляли к первому,
   * зато закрывали угол экрана и уезжали за край.
   */
  function show(body: { title: string; description?: string; icon: string; color: 'success' | 'info' | 'error' }) {
    const same = toast.toasts.value.find(
      (x) => x.open && x.title === body.title && x.description === body.description
    )
    if (same) toast.remove(same.id)
    toast.add({
      ...body,
      // Полоска обратного отсчёта в один пиксель: она подсказывает, сколько
      // тост ещё провисит, и не притворяется содержимым. Прежняя занимала всю
      // ширину и читалась как элемент интерфейса.
      progress: { ui: { base: 'h-px rounded-none bg-transparent', indicator: 'rounded-none' } },
    })
  }

  return {
    ok(title?: string, description?: string) {
      show({
        title: title || t('toast-success'),
        description,
        icon: 'i-lucide-check',
        color: 'success',
      })
    },

    /** Нейтральное уведомление о начатом действии: скачивание, копирование. */
    info(title: string, description?: string) {
      show({ title, description, icon: 'i-lucide-info', color: 'info' })
    },

    fail(err: unknown, title?: string) {
      console.error(err)
      show({
        title: title || t('toast-error'),
        description: describe(err, t),
        icon: 'i-lucide-triangle-alert',
        color: 'error',
      })
    },
  }
}

/** Достаёт текст ошибки мастера; `$fetch` прячет тело ответа в `data`. */
function describe(err: unknown, t: (key: string, args?: Record<string, string | number>) => string): string {
  // Отказ по форме объясняем полями, а не общим «request validation failed»:
  // из него не видно, что именно исправлять.
  const fields = apiErrorFields(err)
  if (fields.length) {
    const number = apiErrorNumber(err)
    const listed = fields.map(f => `${f.field}: ${f.message}`).join('; ')
    return number ? `${listed} (${number})` : listed
  }

  const fromBody = apiErrorLabel(err)
  if (fromBody) return fromBody
  const status = apiErrorStatus(err)
  if (status === 401 || status === 403) return t('auth-session-expired')
  if (status === 429) return t('notif-rate-limited')
  if (status) return t('notif-server-error', { reason: String(status) })
  return (err as { message?: string })?.message || t('toast-error')
}
