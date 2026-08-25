<script setup lang="ts">
/**
 * Быстрый переход по Cmd+K. Разделы — из общего с сайдбаром источника: раньше
 * здесь лежал свой укороченный список, английскими подписями и без проверки
 * прав, так что половина строк вела в 403.
 */
import type { SpotItem, SpotKind } from '~/composables/useSpotlightSources'

const { t } = useT()
const router = useRouter()
const open = useState('noro-spotlight-open', () => false)
const { query, parsed, results, searching, searchUsers, loadServers, loadRecent, remember } =
    useSpotlight()

const selectedIndex = ref(0)
const inputEl = ref<HTMLInputElement | null>(null)

const GROUP_TITLES: Record<SpotKind, string> = {
    nav: 'spotlight-group-nav',
    user: 'spotlight-group-users',
    server: 'spotlight-group-servers',
    action: 'spotlight-group-actions',
}

/** Подпись группы ставится только у первой строки подряд идущих одинаковых. */
function groupLabel(idx: number) {
    const item = results.value[idx]
    if (!item) return null
    if (idx && results.value[idx - 1]?.kind === item.kind) return null
    return t(GROUP_TITLES[item.kind])
}

// Поиск игроков идёт на сервер, поэтому с задержкой: иначе запрос на букву.
let timer: ReturnType<typeof setTimeout> | undefined
watch(
    () => parsed.value.text,
    (text) => {
        clearTimeout(timer)
        timer = setTimeout(() => searchUsers(text), 200)
    },
)

watch(results, () => {
    selectedIndex.value = 0
})

watch(open, (val) => {
    if (!val) return
    query.value = ''
    selectedIndex.value = 0
    loadRecent()
    loadServers()
    nextTick(() => inputEl.value?.focus())
})

async function choose(item: SpotItem) {
    remember(item.id)
    open.value = false
    if (item.run) await item.run()
    else if (item.to) router.push(item.to)
}

function onGlobalKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault()
        open.value = !open.value
        return
    }
    if (!open.value) return
    const total = Math.max(1, results.value.length)
    if (e.key === 'Escape') open.value = false
    else if (e.key === 'ArrowDown') {
        e.preventDefault()
        selectedIndex.value = (selectedIndex.value + 1) % total
    } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        selectedIndex.value = (selectedIndex.value - 1 + total) % total
    } else if (e.key === 'Enter' && results.value[selectedIndex.value]) {
        e.preventDefault()
        choose(results.value[selectedIndex.value]!)
    }
}

onMounted(() => window.addEventListener('keydown', onGlobalKeydown))
onBeforeUnmount(() => {
    clearTimeout(timer)
    window.removeEventListener('keydown', onGlobalKeydown)
})
</script>

<template>
    <Teleport to="body">
        <div
            v-if="open"
            class="fixed inset-0 z-50 flex items-start justify-center bg-black/70 p-4 pt-20"
            @click.self="open = false"
        >
            <div
                class="noro-panel w-full max-w-xl overflow-hidden border border-[var(--noro-border)] shadow-2xl"
            >
                <div
                    class="flex items-center gap-3 border-b border-[var(--noro-border)] bg-[var(--noro-bg-deep)] px-4 py-3"
                >
                    <UIcon
                        :name="searching ? 'i-lucide-loader-circle' : 'i-lucide-search'"
                        class="size-5 shrink-0 text-[var(--noro-muted)]"
                        :class="searching && 'animate-spin'"
                    />
                    <input
                        ref="inputEl"
                        v-model="query"
                        type="text"
                        class="w-full bg-transparent text-sm text-[var(--noro-text)] placeholder-[var(--noro-muted)] focus:outline-none"
                        :placeholder="t('spotlight-placeholder')"
                    />
                    <kbd
                        class="hidden rounded border border-[var(--noro-border)] px-2 py-1 font-mono text-[10px] text-[var(--noro-muted)] sm:inline-block"
                    >
                        ESC
                    </kbd>
                </div>

                <div v-if="results.length" class="noro-scroll max-h-96 overflow-y-auto p-2">
                    <template v-for="(item, idx) in results" :key="item.id">
                        <div
                            v-if="groupLabel(idx)"
                            class="px-3 pb-1 pt-3 text-[10px] font-black uppercase tracking-wider text-[var(--noro-muted)]"
                        >
                            {{ groupLabel(idx) }}
                        </div>
                        <SpotlightRow
                            :item="item"
                            :active="idx === selectedIndex"
                            @click="choose(item)"
                        />
                    </template>
                </div>

                <div v-else class="p-6 text-center text-xs text-[var(--noro-muted)]">
                    {{ t('spotlight-no-results', { query }) }}
                </div>

                <div
                    class="flex items-center justify-between border-t border-[var(--noro-border)] bg-[var(--noro-bg-deep)] px-4 py-2 text-[10px] text-[var(--noro-muted)]"
                >
                    <span>{{ t('spotlight-nav-instructions') }}</span>
                    <span>{{ t('spotlight-prefixes') }}</span>
                </div>
            </div>
        </div>
    </Teleport>
</template>
