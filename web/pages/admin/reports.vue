<script setup lang="ts">
interface ReportRow {
  id: string
  reporter_username: string
  target_username: string
  server_id: string
  reason: string
  world: string
  x: number
  y: number
  z: number
  status: string
  created_at: string
  resolved_at: string | null
}

const auth = useAuth()
const { t } = useT()
const notify = useNotify()
await auth.loadMe()

const reports = ref<ReportRow[]>([])
const openOnly = ref(true)
const pending = ref(false)

async function load() {
  pending.value = true
  try {
    reports.value = await auth.request<ReportRow[]>(`/api/admin/reports?open_only=${openOnly.value}`)
  } catch (e) {
    notify.fail(e, 'Failed to load reports')
  } finally {
    pending.value = false
  }
}

async function resolveReport(id: string) {
  try {
    await auth.request(`/api/admin/reports/${id}/resolve`, { method: 'PUT' })
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  }
}

watch(openOnly, () => load())
onMounted(() => load())
</script>

<template>
  <NoroShell :title="t('admin-reports-title')" :subtitle="t('admin-reports-subtitle')">
    <template #actions>
      <div class="flex items-center gap-4">
        <label class="flex items-center gap-2 text-xs text-[var(--noro-text)] cursor-pointer">
          <input v-model="openOnly" type="checkbox" class="accent-[var(--noro-magenta)]">
          <span>{{ t('admin-reports-open-only') }}</span>
        </label>
        <AtomButton icon="i-lucide-refresh-cw" variant="dark" :loading="pending" @click="load()">
          {{ t('cabinet-apps-refresh') }}
        </AtomButton>
      </div>
    </template>

    <div v-if="reports.length" class="grid gap-3">
      <div
        v-for="rep in reports"
        :key="rep.id"
        class="noro-panel p-4 flex flex-wrap items-center justify-between gap-4"
      >
        <div class="space-y-1">
          <div class="flex items-center gap-2 text-sm font-bold text-[var(--noro-cream)]">
            <span class="text-red-400">@{{ rep.target_username }}</span>
            <span class="text-xs text-[var(--noro-muted)]">от @{{ rep.reporter_username }}</span>
            <span
              class="px-2 py-0.5 text-[10px] rounded uppercase font-bold"
              :class="rep.status === 'open' ? 'bg-amber-500/20 text-amber-400' : 'bg-emerald-500/20 text-emerald-400'"
            >
              {{ rep.status }}
            </span>
          </div>
          <p class="text-xs text-[var(--noro-text)] font-mono">{{ rep.reason }}</p>
          <div class="text-[11px] text-[var(--noro-muted)]">
            📍 {{ rep.world }} [{{ Math.round(rep.x) }}, {{ Math.round(rep.y) }}, {{ Math.round(rep.z) }}]
            · {{ new Date(rep.created_at).toLocaleString() }}
          </div>
        </div>

        <AtomButton
          v-if="rep.status === 'open'"
          variant="primary"
          icon="i-lucide-check-circle"
          @click="resolveReport(rep.id)"
        >
          {{ t('admin-reports-resolve') }}
        </AtomButton>
      </div>
    </div>

    <EmptyState
      v-else-if="!pending"
      icon="i-lucide-flag"
      :title="t('admin-reports-empty-title')"
      :text="t('admin-reports-empty-text')"
    />
  </NoroShell>
</template>
