<script setup lang="ts">
interface DayActivity {
  day: string
  minutes: number
}

const auth = useAuth()
const { t } = useT()

const days = ref<DayActivity[]>([])
const loading = ref(false)

async function load() {
  loading.value = true
  try {
    days.value = await auth.request<DayActivity[]>('/api/me/activity-heatmap')
  } catch (e) {
    console.error('Failed to load activity heatmap', e)
  } finally {
    loading.value = false
  }
}

const maxMinutes = computed(() => {
  if (!days.value.length) return 1
  return Math.max(...days.value.map(d => d.minutes), 1)
})

function colorClass(minutes: number) {
  if (!minutes) return 'bg-[var(--noro-input)] border-[var(--noro-border)]'
  const ratio = minutes / maxMinutes.value
  if (ratio < 0.25) return 'bg-emerald-900/60 border-emerald-700/60'
  if (ratio < 0.5) return 'bg-emerald-700/80 border-emerald-500/80'
  if (ratio < 0.75) return 'bg-emerald-500 border-emerald-400'
  return 'bg-emerald-400 border-emerald-300'
}

onMounted(() => load())
</script>

<template>
  <section class="noro-panel p-6 space-y-4">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-bold text-[var(--noro-cream)] flex items-center gap-2">
        <UIcon name="i-lucide-calendar-days" class="size-4 text-[var(--noro-blue)]" />
        {{ t('cabinet-activity-title') }}
      </h2>
      <span class="text-xs text-[var(--noro-muted)]">{{ t('cabinet-activity-days', { count: 365 }) }}</span>
    </div>

    <div v-if="days.length" class="flex flex-wrap gap-1.5 p-2 rounded bg-[var(--noro-bg-deep)] overflow-x-auto max-h-40">
      <div
        v-for="d in days"
        :key="d.day"
        class="size-3.5 rounded-sm border transition hover:scale-125"
        :class="colorClass(d.minutes)"
        :title="`${d.day}: ${d.minutes}`"
      />
    </div>

    <EmptyState
      v-else-if="!loading"
      icon="i-lucide-calendar"
      :title="t('cabinet-activity-empty-title')"
      :text="t('cabinet-activity-empty-text')"
    />
  </section>
</template>
