<script setup lang="ts">
/** Очередь разборов: кого разбирают, сколько на него жалоб и кто ведёт. */
const auth = useAuth()
const { t } = useT()
await auth.loadMe()

const { cases, openOnly, search, pending, refresh, page, pages, total, perPage, goTo }
  = useCaseQueue()

const STATUS_TONE: Record<string, string> = {
  open: 'text-amber-400',
  in_review: 'text-[var(--noro-blue)]',
  resolved: 'text-emerald-400',
  rejected: 'text-[var(--noro-muted)]',
}
</script>

<template>
  <NoroShell :title="t('admin-cases-title')" :subtitle="t('admin-cases-subtitle')">
    <template #actions>
      <AtomButton icon="i-lucide-refresh-cw" variant="dark" :loading="pending" @click="refresh()">
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
    </template>

    <section class="noro-panel mb-4 flex flex-wrap items-center gap-4 p-3">
      <label class="relative min-w-0 flex-1">
        <UIcon
          name="i-lucide-search"
          class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[var(--noro-muted)]"
        />
        <input v-model="search" class="noro-input w-full !pl-10" :placeholder="t('admin-cases-search')">
      </label>
      <label class="flex shrink-0 cursor-pointer items-center gap-2 text-sm font-bold">
        <input v-model="openOnly" type="checkbox" class="accent-[var(--noro-magenta)]">
        <span>{{ t('admin-cases-open-only') }}</span>
      </label>
      <span class="shrink-0 text-xs text-[var(--noro-muted)]">{{ total }}</span>
    </section>

    <div v-if="cases.length" class="grid gap-3">
      <NuxtLink
        v-for="item in cases"
        :key="item.id"
        :to="adminLink.case(item.id)"
        class="noro-panel flex flex-wrap items-center gap-4 p-4 transition hover:border-[var(--noro-blue)]"
      >
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <span class="font-mono text-xs text-[var(--noro-muted)]">{{ caseNumber(item.number) }}</span>
            <span class="font-bold text-white">{{ item.target_name || '—' }}</span>
            <span class="noro-label">{{ item.server_name || '—' }}</span>
          </div>
          <p class="mt-1 text-xs text-[var(--noro-muted)]">
            {{ t('admin-cases-reports', { reports: item.reports_count, people: item.reporters_count }) }}
            · {{ relativeDateT(item.last_report_at || item.opened_at, t) }}
          </p>
        </div>

        <div class="text-right">
          <div class="noro-label">{{ t('admin-cases-status') }}</div>
          <div class="text-sm font-bold" :class="STATUS_TONE[item.status]">
            {{ t(`admin-cases-status-${item.status}`) }}
          </div>
        </div>

        <div class="w-40 text-right">
          <div class="noro-label">{{ t('admin-cases-claimed-by') }}</div>
          <div class="truncate text-sm text-[var(--noro-text)]">{{ item.claimed_by_name || '—' }}</div>
        </div>
      </NuxtLink>
    </div>

    <EmptyState
      v-else-if="!pending"
      icon="i-lucide-gavel"
      :title="search ? t('paging-empty') : t('admin-cases-empty-title')"
      :text="search ? '' : t('admin-cases-empty-text')"
    />

    <NoroPager :page="page" :pages="pages" :total="total" :per-page="perPage" @go="goTo" />
  </NoroShell>
</template>
