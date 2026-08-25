<script setup lang="ts">
/**
 * Админ: OAuth2-приложения инстанса.
 *
 * Сверху — выключатели и очередь модерации: заходят сюда обычно из-за них.
 * Дальше список с фильтром по состоянию.
 */

import type { AdminApp, AppStatus, ScopeInfo } from '~/types/apps'

const auth = useAuth()
const notify = useNotify()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
await auth.loadMe()

const apps = ref<AdminApp[]>([])
const scopes = ref<ScopeInfo[]>([])
const appsEnabled = ref(true)
const creationEnabled = ref(true)
const pendingCount = ref(0)
const loading = ref(true)
const filter = ref<AppStatus | ''>('')

/**
 * Свёрнутость выключателей — в куке, а не в состоянии страницы.
 *
 * Трогают их редко, а место наверху они занимают всегда; выбор должен пережить
 * перезагрузку, иначе сворачивать бессмысленно. Свёрнуто по умолчанию: обычно
 * сюда заходят за очередью модерации.
 */
const collapsed = useCookie<boolean>('noro-apps-toggles-collapsed', {
  sameSite: 'lax',
  default: () => true,
})
const togglesOpen = computed({
  get: () => !collapsed.value,
  set: (open: boolean) => { collapsed.value = !open },
})

const FILTERS: { id: AppStatus | ''; label: string }[] = [
  { id: '', label: 'admin-apps-filter-all' },
  { id: 'pending', label: 'app-status-pending' },
  { id: 'approved', label: 'app-status-approved' },
  { id: 'rejected', label: 'app-status-rejected' },
  { id: 'suspended', label: 'app-status-suspended' },
]

async function load() {
  loading.value = true
  try {
    const res = await auth.request<{
      items: AdminApp[]
      scopes: ScopeInfo[]
      pending: number
      apps_enabled: boolean
      creation_enabled: boolean
    }>('/api/admin/oauth-apps', { query: filter.value ? { status: filter.value } : {} })
    apps.value = res.items
    scopes.value = res.scopes
    pendingCount.value = res.pending
    appsEnabled.value = res.apps_enabled
    creationEnabled.value = res.creation_enabled
  } catch (e) {
    notify.fail(e)
  } finally {
    loading.value = false
  }
}

async function toggle(body: { apps_enabled?: boolean; creation_enabled?: boolean }) {
  try {
    const res = await auth.request<{ apps_enabled: boolean; creation_enabled: boolean }>(
      '/api/admin/oauth-apps/settings',
      { method: 'PUT', body },
    )
    appsEnabled.value = res.apps_enabled
    creationEnabled.value = res.creation_enabled
    notify.ok()
  } catch (e) {
    notify.fail(e)
    await load()
  }
}

watch(filter, load)
onMounted(load)
</script>

<template>
  <NoroShell :title="t('admin-apps-title')" :subtitle="t('admin-apps-subtitle')">
    <template #actions>
      <AtomButton variant="dark" icon="i-lucide-rotate-cw" size="sm" @click="load" />
    </template>

    <div class="grid gap-4">
      <NoroCard :title="t('admin-apps-toggles')" :subtitle="t('admin-apps-toggles-lead')" icon="i-lucide-power">
        <template #actions>
          <AtomButton
            variant="dark"
            size="sm"
            :icon="togglesOpen ? 'i-lucide-chevron-up' : 'i-lucide-chevron-down'"
            @click="togglesOpen = !togglesOpen"
          />
        </template>

        <div v-if="togglesOpen" class="grid gap-3">
          <label class="flex items-center justify-between gap-4 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3">
            <span>
              <span class="block text-sm font-bold text-[var(--noro-text)]">{{ t('admin-apps-enabled') }}</span>
              <span class="block text-xs text-[var(--noro-muted)]">{{ t('admin-apps-enabled-hint') }}</span>
            </span>
            <input
              v-model="appsEnabled"
              type="checkbox"
              class="size-5 shrink-0 cursor-pointer accent-[var(--noro-magenta)]"
              :disabled="!can('noro.admin.oauth.manage')"
              @change="toggle({ apps_enabled: appsEnabled })"
            >
          </label>

          <label class="flex items-center justify-between gap-4 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3">
            <span>
              <span class="block text-sm font-bold text-[var(--noro-text)]">{{ t('admin-apps-creation') }}</span>
              <span class="block text-xs text-[var(--noro-muted)]">{{ t('admin-apps-creation-hint') }}</span>
            </span>
            <input
              v-model="creationEnabled"
              type="checkbox"
              class="size-5 shrink-0 cursor-pointer accent-[var(--noro-magenta)]"
              :disabled="!can('noro.admin.oauth.manage') || !appsEnabled"
              @change="toggle({ creation_enabled: creationEnabled })"
            >
          </label>
        </div>
      </NoroCard>

      <div class="noro-scroll flex gap-1 overflow-x-auto">
        <button
          v-for="f in FILTERS"
          :key="f.id"
          type="button"
          class="flex shrink-0 items-center gap-2 rounded-[var(--noro-r-sm)] px-3 py-2 text-xs font-bold uppercase tracking-wider transition"
          :class="filter === f.id
            ? 'bg-[var(--noro-panel-2)] text-[var(--noro-cream)]'
            : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
          @click="filter = f.id"
        >
          {{ t(f.label) }}
          <span
            v-if="f.id === 'pending' && pendingCount"
            class="rounded-full bg-[var(--noro-magenta)] px-1.5 text-[10px] text-[var(--noro-white)]"
          >{{ pendingCount }}</span>
        </button>
      </div>

      <div v-if="loading" class="flex justify-center py-12">
        <UIcon name="i-lucide-loader-2" class="size-6 animate-spin text-[var(--noro-blue)]" />
      </div>

      <div v-else class="grid gap-3">
        <AdminAppCard
          v-for="app in apps"
          :key="app.id"
          :app="app"
          :scopes="scopes"
          :can-manage="can('noro.admin.oauth.manage')"
          @changed="load"
        />
        <EmptyState
          v-if="!apps.length"
          icon="i-lucide-app-window"
          :title="t('admin-apps-none')"
          :text="t('admin-apps-none-hint')"
        />
      </div>
    </div>
  </NoroShell>
</template>
