/**
 * Parsing master rejections. Nuxt picks `utils/` up on its own.
 *
 * The master answers with `{"error": {"code", "message"}}` — see
 * `AppError::into_response`. The code is machine-readable and stable, so logic
 * branches on it; the message is human text and gets shown as-is.
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

export interface ApiFieldError {
  field: string
  code: string
  message: string
}

/**
 * Failure number from the master's registry (`error_codes.rs`) — the thing a
 * player quotes over the phone or off a screenshot. A slug wouldn't survive
 * that trip.
 */
export function apiErrorNumber(err: unknown): number | null {
  const n = (envelope(err) as { number?: unknown } | null)?.number
  return typeof n === 'number' ? n : null
}

/** Message with the number in brackets — the form the user sees. */
export function apiErrorLabel(err: unknown): string | null {
  const message = apiErrorMessage(err)
  if (!message) return null
  const number = apiErrorNumber(err)
  return number ? `${message} (${number})` : message
}

export function apiErrorCode(err: unknown): string | null {
  const code = envelope(err)?.code
  return typeof code === 'string' ? code : null
}

/**
 * Fields the master refused. Empty when the failure isn't about the form.
 *
 * Only sent with `validation_error` — see `api::validate` on the master.
 */
export function apiErrorFields(err: unknown): ApiFieldError[] {
  const details = (envelope(err) as { details?: unknown } | null)?.details
  if (!Array.isArray(details)) return []
  return details.filter(
    (d): d is ApiFieldError =>
      typeof d === 'object' && d !== null && typeof (d as ApiFieldError).field === 'string',
  )
}

export function apiFieldMessages(err: unknown): Record<string, string> {
  const out: Record<string, string> = {}
  for (const d of apiErrorFields(err)) {
    // First error per field wins — two messages don't fit under one label.
    if (!(d.field in out)) out[d.field] = d.message
  }
  return out
}

/** Failure text to show the user. `null` means the master explained nothing. */
export function apiErrorMessage(err: unknown): string | null {
  const message = envelope(err)?.message
  if (typeof message === 'string' && message) return message

  // Not our envelope: a raw string body, or `{message}` from a proxy or Nuxt.
  const data = (err as FetchLike)?.data
  if (typeof data === 'string' && data) return data
  if (typeof data === 'object' && data) {
    const plain = (data as { message?: unknown }).message
    if (typeof plain === 'string' && plain) return plain
  }
  return null
}

export function apiErrorStatus(err: unknown): number | null {
  const e = err as FetchLike
  return e?.statusCode ?? e?.status ?? null
}
