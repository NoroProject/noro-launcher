/**
 * Список с серверными поиском и пагинацией.
 *
 * Раньше страницы тянули «побольше и сразу» — `?limit=500` — и фильтровали
 * массив у себя. Пока записей мало, разницы не видно; дальше список молча
 * обрезается, и человек уверен, что нужного просто нет. Поиск и счёт строк —
 * работа базы, здесь только запрос и состояние.
 */
import type { Page } from '~/types/paging'
import { EMPTY_PAGE } from '~/types/paging'

export interface PagedOptions {
    /** Строк на странице. */
    perPage?: number
    /** Дополнительные параметры — фильтры конкретного списка. */
    params?: () => Record<string, string | undefined>
}

/** Пауза перед запросом: иначе уходит по запросу на каждую букву. */
const DEBOUNCE_MS = 250

export function usePagedList<T>(key: string, path: string, options: PagedOptions = {}) {
    const auth = useAuth()
    const perPage = options.perPage ?? 25

    /** То, что человек печатает прямо сейчас. */
    const search = ref('')
    /** То, что уже ушло в запрос. */
    const applied = ref('')
    const page = ref(1)

    let timer: ReturnType<typeof setTimeout> | undefined
    watch(search, (value) => {
        clearTimeout(timer)
        timer = setTimeout(() => {
            applied.value = value
            // Третья страница выдачи по прежнему запросу к новому отношения не
            // имеет — почти всегда она просто пустая.
            page.value = 1
        }, DEBOUNCE_MS)
    })
    onScopeDispose(() => clearTimeout(timer))

    const url = computed(() => {
        const params = new URLSearchParams({
            limit: String(perPage),
            offset: String((page.value - 1) * perPage),
        })
        if (applied.value.trim()) params.set('q', applied.value.trim())
        for (const [k, v] of Object.entries(options.params?.() ?? {})) {
            if (v !== undefined && v !== '') params.set(k, v)
        }
        return `${path}?${params.toString()}`
    })

    /**
     * Отказ пересобирается с адресом запроса.
     *
     * Мастер сообщает только про сам разбор — «invalid type: string "25"», — но
     * молчит о том, какая это была ручка и что за строка. На странице с
     * несколькими списками по такому тексту искать нечего, а в SSR-выдаче он
     * вообще приезжает без контекста.
     */
    async function fetchPage() {
        try {
            return await auth.request<Page<T>>(url.value)
        } catch (e) {
            const detail = humanError(e)
            throw new Error(`${detail} — запрос ${url.value}`)
        }
    }

    const { data, pending, error, refresh } = useAsyncData<Page<T>>(key, fetchPage, {
        default: () => EMPTY_PAGE as Page<T>,
        watch: [url],
    })

    const items = computed(() => data.value?.items ?? [])
    const total = computed(() => data.value?.total ?? 0)
    const pages = computed(() => Math.max(1, Math.ceil(total.value / perPage)))

    function goTo(next: number) {
        page.value = Math.min(Math.max(1, next), pages.value)
    }

    /** Сбросить и поиск, и фильтры к первой странице. */
    function reset() {
        search.value = ''
        applied.value = ''
        page.value = 1
    }

    return {
        search,
        page,
        perPage,
        items,
        total,
        pages,
        pending,
        error,
        refresh,
        goTo,
        reset,
    }
}
