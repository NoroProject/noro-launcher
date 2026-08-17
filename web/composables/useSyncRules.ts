/**
 * Правила синхронизации путей сборки — прямо в файловом менеджере.
 *
 * Мастер хранит два списка: unmanaged_paths и user_managed_paths. Отсюда ровно
 * три состояния, как права на файл: синхронизируется, не трогается, отдан
 * игроку. Правило на папке (путь с косой чертой на конце) наследуется вниз,
 * а своё правило файла перебивает родительское — как более точное.
 */
export type SyncMode = 'sync' | 'ignored' | 'user'

export interface RuleState {
  mode: SyncMode
  /** Путь правила, от которого унаследован режим; пусто — правило своё. */
  from: string
}

export const MODE_ORDER: SyncMode[] = ['sync', 'ignored', 'user']

export const MODE_LABEL: Record<SyncMode, string> = {
  sync: 'S',
  ignored: 'I',
  user: 'U',
}

export function useSyncRules(buildId: string) {
  const auth = useAuth()
  const { t } = useT()

  const MODE_HINT = computed<Record<SyncMode, string>>(() => ({
    sync: t('admin-fm-sync-synced'),
    ignored: t('admin-fm-sync-ignored'),
    user: t('admin-fm-sync-user'),
  }))

  const ignored = ref<string[]>([])
  const user = ref<string[]>([])
  const saving = ref(false)
  /** Загрузка состоялась. Пока false, списки пустые не потому, что правил нет. */
  const loaded = ref(false)

  async function load() {
    const res = await auth.request<{
      build?: { unmanaged_paths?: string[]; user_managed_paths?: string[] }
      unmanaged_paths?: string[]
      user_managed_paths?: string[]
    }>(`/api/admin/builds/${buildId}`)
    const target = res.build || res
    ignored.value = target.unmanaged_paths || []
    user.value = target.user_managed_paths || []
    loaded.value = true
  }

  /** Самое длинное правило, накрывающее путь: точное совпадение или папка выше. */
  function ruleFor(path: string): RuleState {
    let best: RuleState = { mode: 'sync', from: '' }
    let bestLen = -1
    const check = (list: string[], mode: SyncMode) => {
      for (const rule of list) {
        const covers = rule === path || (rule.endsWith('/') && path.startsWith(rule))
        if (covers && rule.length > bestLen) {
          best = { mode, from: rule === path ? '' : rule }
          bestLen = rule.length
        }
      }
    }
    check(ignored.value, 'ignored')
    check(user.value, 'user')
    return best
  }

  /** Поставить режим на путь, убрав прежнее собственное правило. */
  function setMode(path: string, mode: SyncMode) {
    ignored.value = ignored.value.filter(p => p !== path)
    user.value = user.value.filter(p => p !== path)
    if (mode === 'ignored') ignored.value = [...ignored.value, path]
    if (mode === 'user') user.value = [...user.value, path]
  }

  /** Клик перебирает режимы по кругу — как chmod по-быстрому. */
  function cycle(path: string) {
    const current = ruleFor(path)
    const next = current.from
      ? current.mode
      : MODE_ORDER[(MODE_ORDER.indexOf(current.mode) + 1) % MODE_ORDER.length]
    setMode(path, next)
  }

  async function save() {
    if (!loaded.value) {
      throw new Error('Sync rules are not loaded yet — refusing to overwrite them')
    }
    saving.value = true
    try {
      await auth.request(`/api/admin/builds/${buildId}/paths`, {
        method: 'PUT',
        body: { unmanaged_paths: ignored.value, user_managed_paths: user.value },
      })
    } finally {
      saving.value = false
    }
  }

  const count = computed(() => ignored.value.length + user.value.length)

  return { ignored, user, saving, loaded, count, MODE_HINT, load, ruleFor, setMode, cycle, save }
}
