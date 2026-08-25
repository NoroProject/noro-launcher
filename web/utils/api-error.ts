/**
 * Разбор отказа мастера. Nuxt подхватывает `utils/` сам.
 *
 * Мастер отвечает `{"error": {"code", "message"}}` — см. `AppError::into_response`.
 * Код машинный и стабильный, по нему ветвится логика; сообщение человеческое,
 * его показывают. Раньше тело было одной строкой, и вид отказа склеивался с
 * причиной («forbidden: step_up_required») — интерфейсу приходилось искать
 * подстроку, а разбор был продублирован в двух местах и успел разойтись.
 */

type FetchLike = {
  data?: unknown
  statusCode?: number
  status?: number
  message?: string
}

function envelope(err: unknown): { code?: unknown; message?: unknown } | null {
  const data = (err as FetchLike)?.data
  if (typeof data !== 'object' || !data) return null
  const error = (data as { error?: unknown }).error
  if (typeof error !== 'object' || !error) return null
  return error as { code?: unknown; message?: unknown }
}

/** Промах в одном поле формы. */
export interface ApiFieldError {
  field: string
  code: string
  message: string
}

/**
 * Номер отказа из реестра мастера (`error_codes.rs`).
 *
 * Его называет человек: «у меня 1307». Слаг для этого не годится — его не
 * продиктуешь по телефону и не запомнишь со скриншота.
 */
export function apiErrorNumber(err: unknown): number | null {
  const n = (envelope(err) as { number?: unknown } | null)?.number
  return typeof n === 'number' ? n : null
}

/** Текст с номером в скобках — то, что видит человек. */
export function apiErrorLabel(err: unknown): string | null {
  const message = apiErrorMessage(err)
  if (!message) return null
  const number = apiErrorNumber(err)
  return number ? `${message} (${number})` : message
}

/** Машинный код отказа, если мастер его прислал. */
export function apiErrorCode(err: unknown): string | null {
  const code = envelope(err)?.code
  return typeof code === 'string' ? code : null
}

/**
 * Поля, которые мастер не принял. Пусто — отказ не про форму.
 *
 * Приходит только у `validation_error`, см. `api::validate` на мастере. Форма
 * по этому списку подсвечивает виноватые поля вместо того, чтобы показывать
 * один абзац под собой.
 */
export function apiErrorFields(err: unknown): ApiFieldError[] {
  const details = (envelope(err) as { details?: unknown } | null)?.details
  if (!Array.isArray(details)) return []
  return details.filter(
    (d): d is ApiFieldError =>
      typeof d === 'object' && d !== null && typeof (d as ApiFieldError).field === 'string',
  )
}

/** Промахи, разложенные по имени поля: то, что нужно форме для подсветки. */
export function apiFieldMessages(err: unknown): Record<string, string> {
  const out: Record<string, string> = {}
  for (const d of apiErrorFields(err)) {
    // Первый промах по полю и есть тот, что показываем: их редко больше одного,
    // а два сообщения в одной подписи не помещаются.
    if (!(d.field in out)) out[d.field] = d.message
  }
  return out
}

/** Текст отказа для показа пользователю. `null` — мастер ничего не объяснил. */
export function apiErrorMessage(err: unknown): string | null {
  const message = envelope(err)?.message
  if (typeof message === 'string' && message) return message

  // Не наш конверт: сырое тело строкой или `{message}` от прокси/Nuxt.
  const data = (err as FetchLike)?.data
  if (typeof data === 'string' && data) return data
  if (typeof data === 'object' && data) {
    const plain = (data as { message?: unknown }).message
    if (typeof plain === 'string' && plain) return plain
  }
  return null
}

/** HTTP-статус отказа, как его отдаёт `$fetch`. */
export function apiErrorStatus(err: unknown): number | null {
  const e = err as FetchLike
  return e?.statusCode ?? e?.status ?? null
}
