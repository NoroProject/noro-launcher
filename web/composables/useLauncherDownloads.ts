export interface LauncherDownload {
  platform: string
  version: string
  size: number
  sha256: string
  url: string
  filename: string
}

/** Человекочитаемые названия платформ мастера. */
const LABELS: Record<string, string> = {
  'windows-x86_64': 'Windows',
  'macos-aarch64': 'macOS (Apple Silicon)',
  'macos-x86_64': 'macOS (Intel)',
  'linux-x86_64': 'Linux',
  'linux-aarch64': 'Linux (ARM)',
}

export function useLauncherDownloads() {
  const auth = useAuth()

  const { data, pending } = useAsyncData('launcher-downloads', () =>
    auth.request<LauncherDownload[]>('/api/launcher/downloads'),
    { default: () => [] as LauncherDownload[] }
  )

  /**
   * Платформа посетителя по user-agent. Точного способа нет, поэтому это лишь
   * подсказка: полный список остаётся рядом, чтобы промах не увёл человека на
   * чужой файл.
   */
  const guessed = computed(() => {
    if (!import.meta.client) return null
    const ua = navigator.userAgent
    const arm = /arm|aarch64/i.test(ua)
    if (/Win/i.test(ua)) return 'windows-x86_64'
    if (/Mac/i.test(ua)) {
      // Apple Silicon не признаётся в user-agent, поэтому опираемся на число
      // ядер: у всех ARM-маков их 8 и больше, у старых Intel — обычно меньше.
      return arm || (navigator.hardwareConcurrency || 0) >= 8 ? 'macos-aarch64' : 'macos-x86_64'
    }
    if (/Linux|X11/i.test(ua)) return arm ? 'linux-aarch64' : 'linux-x86_64'
    return null
  })

  const primary = computed(() =>
    data.value.find(d => d.platform === guessed.value) || data.value[0] || null
  )

  const others = computed(() => data.value.filter(d => d !== primary.value))

  function label(platform: string) {
    return LABELS[platform] || platform
  }

  /** Стор адресуется по хешу, поэтому имя файла передаётся отдельно. */
  function href(item: LauncherDownload) {
    return `${item.url}?name=${encodeURIComponent(item.filename)}`
  }

  function size(bytes: number) {
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  }

  return { downloads: data, pending, primary, others, label, href, size }
}
