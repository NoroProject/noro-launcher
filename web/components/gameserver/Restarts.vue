<script setup lang="ts">
import type { NewRestartSchedule, RestartSchedule } from '~/types/restart'

/**
 * Расписание рестартов игрового server'а.
 *
 * В отличие от соседних вкладок не зависит от враппера: расписания живут на
 * мастере, и заводить их можно, пока сервер выключен, — планировщик там же и
 * крутится.
 */
const props = defineProps<{ gameServerId: string }>()

const auth = useAuth()
const canEdit = computed(() => auth.hasPermission('noro.admin.servers.edit'))

const rows = ref<RestartSchedule[]>([])
const busy = ref(false)
const error = ref<string | null>(null)

async function refresh() {
    busy.value = true
    error.value = null
    try {
        rows.value = await auth.requestList<RestartSchedule>(`/api/admin/restarts?game_server_id=${props.gameServerId}`,
        )
    } catch (e: unknown) {
        error.value = e instanceof Error ? e.message : String(e)
    } finally {
        busy.value = false
    }
}

async function create(body: NewRestartSchedule) {
    busy.value = true
    error.value = null
    try {
        await auth.request('/api/admin/restarts', { method: 'POST', body })
        await refresh()
    } catch (e: unknown) {
        error.value = e instanceof Error ? e.message : String(e)
    } finally {
        busy.value = false
    }
}

async function remove(id: string) {
    busy.value = true
    try {
        await auth.request(`/api/admin/restarts/${id}`, { method: 'DELETE' })
        await refresh()
    } catch (e: unknown) {
        error.value = e instanceof Error ? e.message : String(e)
    } finally {
        busy.value = false
    }
}

/** Человеческое описание того, когда сработает. */
function when(row: RestartSchedule) {
    if (row.at_times?.length) return `at ${row.at_times.join(', ')}`
    if (row.interval_minutes) return `every ${row.interval_minutes / 60}h`
    return row.cron_expr ? `cron ${row.cron_expr}` : '—'
}

function stamp(value?: string | null) {
    return value ? new Date(value).toLocaleString() : '—'
}

onMounted(refresh)
</script>

<template>
    <div class="noro-panel p-5 flex flex-col h-full min-h-0 gap-4 overflow-y-auto">
        <div class="flex shrink-0 items-center justify-between gap-4">
            <div>
                <h3 class="text-sm font-bold uppercase tracking-wider text-[var(--noro-text)]">
                    Restart schedule
                </h3>
                <p class="text-xs text-[var(--noro-muted)]">
                    Players get a countdown; the master calls the wrapper when it runs out.
                </p>
            </div>
            <AtomButton variant="dark" icon="i-lucide-refresh-cw" :disabled="busy" @click="refresh">
                Refresh
            </AtomButton>
        </div>

        <div v-if="error" class="shrink-0 rounded-lg border border-red-900/40 bg-red-950/20 p-3 text-xs text-red-400">
            {{ error }}
        </div>

        <GameserverRestartForm v-if="canEdit" :game-server-id="gameServerId" :busy="busy" @submit="create" />

        <p v-if="!rows.length" class="py-6 text-center text-xs text-[var(--noro-muted)]">
            No schedules yet — the server restarts only by hand.
        </p>

        <ul v-else class="space-y-2">
            <li
                v-for="row in rows"
                :key="row.id"
                class="flex items-center justify-between gap-4 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] p-3"
            >
                <div class="min-w-0 space-y-1">
                    <p class="font-mono text-xs text-[var(--noro-text)]">{{ when(row) }}</p>
                    <p class="text-[11px] text-[var(--noro-muted)]">
                        warn {{ row.notice_minutes }}m ·
                        {{ row.online_policy === 'defer' ? `wait up to ${row.max_defer_minutes}m` : 'restart anyway' }} ·
                        next {{ stamp(row.next_run_at) }} · last {{ stamp(row.last_run_at) }}
                    </p>
                </div>
                <AtomButton
                    v-if="canEdit"
                    variant="ghost"
                    icon="i-lucide-trash-2"
                    class="shrink-0 text-[var(--noro-danger)]"
                    :disabled="busy"
                    @click="remove(row.id)"
                >
                    Delete
                </AtomButton>
            </li>
        </ul>
    </div>
</template>
