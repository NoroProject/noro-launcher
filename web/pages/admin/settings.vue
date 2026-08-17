<script setup lang="ts">
import {
  SECTIONS,
  SETTING_LABELS,
  type DiagnosticCheck,
  type SettingsResponse,
} from '~/types/settings'

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
const notify = useNotify()
await auth.loadMe()

const data = ref<SettingsResponse | null>(null)
const checks = ref<DiagnosticCheck[]>([])
const draft = ref<Record<string, string>>({})
const pending = ref(false)
const restartRequired = ref(false)

async function load() {
  pending.value = true
  try {
    data.value = await auth.request<SettingsResponse>('/api/admin/settings')
    draft.value = Object.fromEntries(data.value.settings.map((s) => [s.key, s.value]))
    checks.value = (
      await auth.request<{ checks: DiagnosticCheck[] }>('/api/admin/diagnostics')
    ).checks
  } catch (e) {
    notify.fail(e, 'Failed to load settings')
  } finally {
    pending.value = false
  }
}

async function save() {
  pending.value = true
  try {
    const editable = Object.fromEntries(
      (data.value?.settings ?? [])
        .filter((s) => !s.from_env)
        .map((s) => [s.key, draft.value[s.key] ?? ''])
    )
    const res = await auth.request<{ changed: number; restart_required?: boolean }>(
      '/api/admin/settings',
      { method: 'PUT', body: { settings: editable } }
    )
    if (res.restart_required) restartRequired.value = true
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  } finally {
    pending.value = false
  }
}

async function exportEnv() {
  try {
    const { env } = await auth.request<{ env: string }>('/api/admin/settings/env')
    const url = URL.createObjectURL(new Blob([env], { type: 'text/plain' }))
    const a = document.createElement('a')
    a.href = url
    a.download = '.env'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e) {
    notify.fail(e)
  }
}

const inSection = (name: string) =>
  (data.value?.settings ?? []).filter((s) => SETTING_LABELS[s.key]?.section === name)

onMounted(() => load())
</script>

<template>
  <NoroShell :title="t('admin-settings-title')" :subtitle="t('admin-settings-subtitle')">
    <template #actions>
      <AtomButton icon="i-lucide-download" variant="dark" @click="exportEnv">{{ t('admin-settings-export-env') }}</AtomButton>
      <AtomButton v-if="can('noro.admin.settings.edit')" icon="i-lucide-save" :loading="pending" @click="save">{{ t('cabinet-save') }}</AtomButton>
    </template>

    <UAlert
      v-if="restartRequired"
      class="mb-4"
      color="warning"
      variant="subtle"
      icon="i-lucide-rotate-ccw"
      :title="t('admin-settings-restart-title')"
      :description="t('admin-settings-restart-desc')"
    />

    <div class="grid gap-4 xl:grid-cols-[1fr_360px]">
      <div class="grid gap-4">
        <section v-for="name in SECTIONS" :key="name" class="noro-panel p-6">
          <h2 class="mb-4 text-lg font-black text-[var(--noro-text)]">
            {{ name === 'General' ? t('admin-settings-sec-general') : name === 'Auth' ? t('admin-settings-sec-auth') : name === 'Storage' ? t('admin-settings-sec-storage') : t('admin-settings-sec-integrations') }}
          </h2>
          <div class="grid gap-4">
            <SettingsField
              v-for="item in inSection(name)"
              :key="item.key"
              v-model="draft[item.key]"
              :item="item"
            />
          </div>
        </section>

        <section class="noro-panel p-6">
          <h2 class="mb-1 text-lg font-black text-[var(--noro-text)]">{{ t('admin-settings-secrets-title') }}</h2>
          <p class="mb-4 text-xs text-[var(--noro-muted)]">
            {{ t('admin-settings-secrets-lead') }}
          </p>
          <ul class="grid gap-2">
            <li
              v-for="(present, name) in data?.secrets ?? {}"
              :key="name"
              class="flex items-center justify-between rounded border border-[var(--noro-border)] bg-[var(--noro-bg)] px-3 py-2 text-xs"
            >
              <code>{{ name }}</code>
              <span :class="present ? 'text-[var(--noro-blue)]' : 'text-[var(--noro-muted)]'">
                {{ present ? t('admin-settings-secret-set') : t('admin-settings-secret-unset') }}
              </span>
            </li>
          </ul>
        </section>
      </div>

      <DiagnosticsPanel :checks="checks" />
    </div>
  </NoroShell>
</template>
