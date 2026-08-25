<script setup lang="ts">
/** Карточка разбора: жалобы, лента действий, срез чата и наказание. */
const route = useRoute()
const auth = useAuth()
const { t } = useT()
await auth.loadMe()

const caseId = computed(() => String(route.params.id))
const { detail, pending, load, act } = useCase(caseId)

const item = computed(() => detail.value?.case)
const reporters = computed(() => detail.value?.reporters || {})

</script>

<template>
  <NoroShell :title="t('admin-case-title')" :subtitle="item?.target_name || ''">
    <template #actions>
      <AtomButton variant="dark" icon="i-lucide-arrow-left" :to="adminLink.cases()">
        {{ t('nav-admin-cases') }}
      </AtomButton>
      <AtomButton icon="i-lucide-refresh-cw" variant="dark" :loading="pending" @click="load()">
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
    </template>

    <div v-if="detail && item" class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(0,24rem)]">
      <div class="grid min-w-0 grid-cols-[minmax(0,1fr)] content-start gap-6">
        <AdminCaseHeader :item="item" :punishments="detail.punishments.length" />

        <section class="noro-panel min-w-0 p-4">
          <div class="noro-label mb-3">{{ t('admin-case-reports') }}</div>
          <ul class="grid gap-3">
            <li v-for="report in detail.reports" :key="report.id" class="grid gap-0.5">
              <div class="flex flex-wrap items-baseline gap-2">
                <span class="font-bold text-white">{{ report.reporter_name || '—' }}</span>
                <AdminReporterTrust :stats="reporters[report.reporter_id]" />
                <time class="ml-auto font-mono text-xs text-[var(--noro-muted)]">{{ clockTime(report.created_at) }}</time>
              </div>
              <p class="text-sm leading-6 text-[var(--noro-text)]">{{ report.reason }}</p>
              <!-- Либо место целиком, либо честное «его нет». `|| 0` показывал
                   «0 0 0» — точку в мире, куда модератор и телепортировался. -->
              <p
                v-if="report.world && report.x !== null && report.y !== null && report.z !== null"
                class="font-mono text-xs text-[var(--noro-muted)]"
              >
                {{ report.world }} · {{ Math.round(report.x) }} {{ Math.round(report.y) }} {{ Math.round(report.z) }}
              </p>
              <p v-else class="font-mono text-xs text-[var(--noro-muted)]">
                {{ t('admin-case-report-no-place') }}
              </p>
            </li>
          </ul>
        </section>

        <section class="noro-panel min-w-0 p-4">
          <div class="noro-label mb-3">{{ t('admin-case-timeline') }}</div>
          <AdminCaseTimeline :events="detail.events" />
        </section>

        <section class="noro-panel min-w-0 p-4">
          <div class="noro-label mb-3">{{ t('admin-case-chat') }}</div>
          <AdminCaseChat :messages="detail.messages" :allowed="detail.chat_allowed" />
        </section>
      </div>

      <div class="grid min-w-0 grid-cols-[minmax(0,1fr)] content-start gap-6">
        <AdminCaseActions :item="item" @act="(path, method, body) => act(path, method, body)" />

        <section class="noro-panel min-w-0 p-4">
          <div class="noro-label mb-3">{{ t('admin-case-punish') }}</div>
          <PunishmentForm :user-id="item.target_id" :case-id="item.id" @created="load(true)" />
        </section>
      </div>
    </div>

    <EmptyState
      v-else-if="!pending"
      icon="i-lucide-gavel"
      :title="t('admin-case-missing-title')"
      :text="t('admin-case-missing-text')"
    />
  </NoroShell>
</template>
