<script setup lang="ts">
const auth = useAuth()
await auth.loadMe()

const { data: stats, pending, refresh, error } = await useAsyncData('admin-stats', () =>
  auth.request<Record<string, number>>('/api/admin/stats')
)

const cards = computed(() => [
  { label: 'Users', value: stats.value?.users ?? 0, icon: 'i-lucide-users', tone: 'blue' as const },
  { label: 'Servers', value: stats.value?.servers ?? 0, icon: 'i-lucide-server', tone: 'green' as const },
  { label: 'Builds', value: stats.value?.builds ?? 0, icon: 'i-lucide-package', tone: 'amber' as const },
  { label: 'Launchers online', value: stats.value?.online_launchers ?? 0, icon: 'i-lucide-radio', tone: 'magenta' as const }
])

const fileStoreGb = computed(() => {
  const bytes = stats.value?.file_store_bytes || 0
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`
})
</script>

<template>
  <NoroShell title="ADMIN" subtitle="Master server operations dashboard">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        Refresh
      </AtomButton>
    </template>

    <UAlert
      v-if="error"
      class="mb-5"
      color="error"
      variant="subtle"
      icon="i-lucide-circle-alert"
      :description="humanError(error)"
    />

    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <MetricCard v-for="card in cards" :key="card.label" v-bind="card" />
    </div>

    <div class="mt-5 grid gap-5 xl:grid-cols-[1fr_420px]">
      <section class="noro-panel p-5">
        <div class="mb-4 flex items-center justify-between">
          <h2 class="font-bold text-[var(--noro-text)]">Data state</h2>
          <UIcon name="i-lucide-database" class="size-5 text-[var(--noro-cream)]" />
        </div>
        <div class="grid gap-3">
          <div class="rounded-lg bg-[var(--noro-input)] p-4 flex items-center justify-between">
            <div>
              <div class="text-xs font-bold uppercase tracking-wider text-[var(--noro-muted)]">FileStore Storage</div>
              <div class="mt-1 text-2xl font-black text-[var(--noro-text)]">{{ fileStoreGb }}</div>
            </div>
            <UIcon name="i-lucide-hard-drive" class="size-8 text-[var(--noro-blue)]" />
          </div>
        </div>
      </section>

      <section class="noro-panel p-5">
        <h2 class="mb-4 font-bold text-[var(--noro-text)]">Quick actions</h2>
        <div class="grid gap-2">
          <QuickAction to="/admin/servers" icon="i-lucide-plus" label="Create server" primary />
          <QuickAction to="/admin/news" icon="i-lucide-newspaper" label="Publish news" />
          <QuickAction to="/admin/launcher" icon="i-lucide-rocket" label="Deploy launcher" />
        </div>
      </section>
    </div>
  </NoroShell>
</template>
