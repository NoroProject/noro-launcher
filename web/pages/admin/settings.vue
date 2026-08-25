<script setup lang="ts">
/**
 * Настройки инстанса.
 *
 * Вкладками, а не простынёй секций: полей десяток, но читаются они группами —
 * брендинг правят раз в полгода, а URL-ы и интеграции живут своей жизнью.
 * Способы входа переехали сюда же со своей страницы: это те же настройки, и
 * отдельным разделом сайдбара они только удлиняли список.
 */

import {
  BRANDING_IMAGES,
  SETTINGS_TABS,
  SETTING_LABELS,
  type DiagnosticCheck,
  type SettingsResponse,
} from '~/types/settings'

const auth = useAuth()
const route = useRoute()
const router = useRouter()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
const notify = useNotify()
await auth.loadMe()

const data = ref<SettingsResponse | null>(null)
const checks = ref<DiagnosticCheck[]>([])
const draft = ref<Record<string, string>>({})
const pending = ref(false)
const restartRequired = ref(false)

// Вкладка в адресе: ссылкой на «настройки» обычно зовут в конкретное место.
const tab = computed({
  get: () => SETTINGS_TABS.find(x => x.id === route.query.tab)?.id ?? SETTINGS_TABS[0]!.id,
  set: (id: string) => router.replace({ query: { ...route.query, tab: id } }),
})
const current = computed(() => SETTINGS_TABS.find(x => x.id === tab.value)!)

const imageKeys = new Set<string>(BRANDING_IMAGES.map(i => i.key))
const settings = computed(() => data.value?.settings ?? [])
/** Поля вкладки. Картинки исключены: они живут в своих карточках с превью. */
const fields = computed(() =>
  settings.value.filter(
    s => SETTING_LABELS[s.key]?.section === current.value.section && !imageKeys.has(s.key),
  ),
)
/** Картинки, которые мастер знает: список ключей у сайта свой, и он может
 *  разъехаться со списком настроек — тогда карточки просто нет. */
const images = computed(() =>
  BRANDING_IMAGES.flatMap((img) => {
    const found = settings.value.find(s => s.key === img.key)
    return found ? [{ ...img, item: found }] : []
  }),
)

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
      settings.value.filter((s) => !s.from_env).map((s) => [s.key, draft.value[s.key] ?? '']),
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

onMounted(() => load())
</script>

<template>
  <NoroShell :title="t('admin-settings-title')" :subtitle="t('admin-settings-subtitle')">
    <template #actions>
      <AtomButton icon="i-lucide-download" variant="dark" @click="exportEnv">{{ t('admin-settings-export-env') }}</AtomButton>
      <AtomButton
        v-if="can('noro.admin.settings.edit') && current.section"
        icon="i-lucide-save"
        :loading="pending"
        @click="save"
      >{{ t('cabinet-save') }}</AtomButton>
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

    <!-- `shrink-0` обязателен: полоса лежит во флекс-колонке страницы, и стоит
         содержимому вкладки перерасти экран, как она сжимается в свою рамку —
         вкладки видно ровно до того, как приедут данные. -->
    <div class="noro-scroll mb-4 flex shrink-0 gap-1 overflow-x-auto border-b border-[var(--noro-border)]">
      <button
        v-for="entry in SETTINGS_TABS"
        :key="entry.id"
        type="button"
        class="flex shrink-0 items-center gap-2 border-b-2 px-4 py-3 text-xs font-bold uppercase tracking-wider transition"
        :class="entry.id === tab
          ? 'border-[var(--noro-cream)] text-[var(--noro-cream)]'
          : 'border-transparent text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
        @click="tab = entry.id"
      >
        <UIcon :name="entry.icon" class="size-4" />
        {{ t(entry.label) }}
      </button>
    </div>

    <AdminAuthMethodsPanel v-if="tab === 'sign-in'" />

    <div v-else-if="tab === 'health'" class="grid gap-4 xl:grid-cols-[1fr_360px]">
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

      <DiagnosticsPanel :checks="checks" />
    </div>

    <section v-else class="noro-panel grid gap-4 p-6">
      <SettingsField
        v-for="entry in fields"
        :key="entry.key"
        v-model="draft[entry.key]"
        :item="entry"
      />

      <AdminSettingsImage
        v-for="img in (tab === 'branding' ? images : [])"
        :key="img.key"
        v-model="draft[img.key]"
        :item="img.item"
        :title="t(img.title)"
        :desc="t(img.desc)"
        :transparent="data?.transparency?.[img.key] === true"
        :can-edit="can('noro.admin.settings.edit')"
        @uploaded="load"
      />
    </section>
  </NoroShell>
</template>
