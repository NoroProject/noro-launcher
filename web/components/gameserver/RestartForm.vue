<script setup lang="ts">
import type { NewRestartSchedule } from '~/types/restart'

/**
 * Форма нового расписания.
 *
 * Три способа задать время — переключателем, а не тремя полями сразу: мастер
 * всё равно возьмёт первый непустой, и показывать одновременно «в 05:00»,
 * «каждые 6 часов» и cron значит обещать сумму, которой не будет.
 */
const props = defineProps<{ gameServerId: string; busy: boolean }>()
const emit = defineEmits<{ submit: [NewRestartSchedule] }>()

type Mode = 'at_times' | 'interval' | 'cron'
const MODES: { id: Mode; label: string }[] = [
    { id: 'at_times', label: 'At times' },
    { id: 'interval', label: 'Every N hours' },
    { id: 'cron', label: 'Cron' },
]

const mode = ref<Mode>('at_times')
const times = ref('05:00')
const intervalHours = ref(6)
const cron = ref('0 5 * * *')
const notice = ref(5)
const policy = ref('warn_and_go')
const maxDefer = ref(30)

/** `HH:MM` через запятую или пробел — как удобнее набрать. */
const parsedTimes = computed(() =>
    times.value.split(/[,\s]+/).map(s => s.trim()).filter(Boolean),
)

const valid = computed(() => {
    if (mode.value === 'at_times') return parsedTimes.value.every(t => /^\d{1,2}:\d{2}$/.test(t)) && parsedTimes.value.length > 0
    if (mode.value === 'interval') return intervalHours.value > 0
    return cron.value.trim().split(/\s+/).length >= 5
})

function submit() {
    if (!valid.value || props.busy) return
    const body: NewRestartSchedule = {
        game_server_id: props.gameServerId,
        notice_minutes: notice.value,
        online_policy: policy.value,
        max_defer_minutes: maxDefer.value,
    }
    if (mode.value === 'at_times') body.at_times = parsedTimes.value
    else if (mode.value === 'interval') body.interval_minutes = intervalHours.value * 60
    else body.cron_expr = cron.value.trim()
    emit('submit', body)
}
</script>

<template>
    <form class="space-y-3 rounded-lg border border-[var(--noro-border)] p-4" @submit.prevent="submit">
        <div class="flex flex-wrap gap-2">
            <button
                v-for="m in MODES"
                :key="m.id"
                type="button"
                class="noro-chip px-3 py-1.5 text-[11px] font-bold uppercase tracking-wider"
                :class="mode === m.id ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
                @click="mode = m.id"
            >{{ m.label }}</button>
        </div>

        <div class="grid gap-3 sm:grid-cols-2">
            <label v-if="mode === 'at_times'" class="space-y-1">
                <span class="noro-label">Times (HH:MM)</span>
                <input v-model="times" type="text" placeholder="05:00, 17:00" class="noro-input w-full text-xs">
            </label>
            <label v-else-if="mode === 'interval'" class="space-y-1">
                <span class="noro-label">Every, hours</span>
                <input v-model.number="intervalHours" type="number" min="1" class="noro-input w-full text-xs">
            </label>
            <label v-else class="space-y-1">
                <span class="noro-label">Cron</span>
                <input v-model="cron" type="text" placeholder="0 5 * * 1-5" class="noro-input w-full text-xs">
            </label>

            <label class="space-y-1">
                <span class="noro-label">Warn players, minutes</span>
                <input v-model.number="notice" type="number" min="0" class="noro-input w-full text-xs">
            </label>

            <label class="space-y-1">
                <span class="noro-label">When players are online</span>
                <NoroSelect v-model="policy" class="w-full">
                    <option value="warn_and_go">Warn and restart anyway</option>
                    <option value="defer">Wait for an empty server</option>
                </NoroSelect>
            </label>

            <label v-if="policy === 'defer'" class="space-y-1">
                <span class="noro-label">…but no longer than, minutes</span>
                <input v-model.number="maxDefer" type="number" min="1" class="noro-input w-full text-xs">
            </label>
        </div>

        <AtomButton variant="primary" icon="i-lucide-plus" type="submit" :disabled="!valid || busy">
            Add schedule
        </AtomButton>
    </form>
</template>
