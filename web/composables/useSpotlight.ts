/**
 * Разбор запроса, ранжирование и недавнее для спотлайта.
 *
 * Раньше выдача была голым `includes` в порядке загрузки: «Резервная копия» и
 * «Копия мастера» шли вперемешку, а сокращение не находило ничего. Теперь
 * порядок задаёт `scoreAny`, а при пустом запросе сверху стоит то, чем реально
 * пользуются.
 */
import type { SpotItem, SpotKind } from '~/composables/useSpotlightSources'

/** Приставки-фильтры: `@` — люди, `#` — серверы, `>` — команды. */
const PREFIXES: Record<string, SpotKind> = { '@': 'user', '#': 'server', '>': 'action' }

const RECENT_KEY = 'noro-spotlight-recent'
/** Пять последних: длиннее список превращается в свалку, а не в подсказку. */
const RECENT_MAX = 5

export function useSpotlight() {
    const sources = useSpotlightSources()
    const query = ref('')
    const recentIds = ref<string[]>([])

    const parsed = computed(() => {
        const raw = query.value.trim()
        const only = PREFIXES[raw[0] ?? '']
        return { only, text: (only ? raw.slice(1) : raw).trim() }
    })

    const pool = computed<SpotItem[]>(() => {
        const all = [
            ...sources.navItems.value,
            ...sources.serverItems.value,
            ...sources.userItems.value,
            ...sources.actionItems.value,
        ]
        const { only } = parsed.value
        return only ? all.filter((i) => i.kind === only) : all
    })

    const results = computed<SpotItem[]>(() => {
        const q = parsed.value.text
        if (!q) {
            const recent = recentIds.value
                .map((id) => pool.value.find((i) => i.id === id))
                .filter((i): i is SpotItem => !!i)
            const rest = pool.value.filter((i) => !recentIds.value.includes(i.id))
            return [...recent, ...rest].slice(0, 8)
        }
        return pool.value
            .map((item) => ({ item, rank: scoreAny(q, item.title, item.subtitle) }))
            .filter((r) => r.rank > 0)
            .sort((a, b) => b.rank - a.rank)
            .slice(0, 15)
            .map((r) => r.item)
    })

    function loadRecent() {
        try {
            recentIds.value = JSON.parse(localStorage.getItem(RECENT_KEY) || '[]')
        } catch {
            recentIds.value = []
        }
    }

    function remember(id: string) {
        recentIds.value = [id, ...recentIds.value.filter((x) => x !== id)].slice(0, RECENT_MAX)
        localStorage.setItem(RECENT_KEY, JSON.stringify(recentIds.value))
    }

    return {
        query,
        parsed,
        results,
        searching: sources.searching,
        searchUsers: sources.searchUsers,
        loadServers: sources.loadServers,
        loadRecent,
        remember,
    }
}
