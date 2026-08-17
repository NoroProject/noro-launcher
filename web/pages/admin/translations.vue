<script setup lang="ts">
import { buildFtl, isMultiline, parseFtl } from '~/composables/use-ftl'

interface LocaleInfo { locale: string, sha1: string }
interface Catalog { locale: string, sha1: string, ftl: string, builtin: string }

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
await auth.loadMe()

const LOCALES = [
  { code: 'en', label: 'English' },
  { code: 'ru', label: 'Русский' }
]

const active = ref('ru')
const busy = ref(false)
const message = ref<string | null>(null)
const error = ref<string | null>(null)
const search = ref('')
const onlyChanged = ref(false)

/** Ключ → встроенный текст. Это эталон: он задаёт, что вообще переводится. */
const builtin = ref(new Map<string, string>())
/** Ключ → переопределение. Пусто = используется встроенный. */
const overrides = reactive(new Map<string, string>())
const savedSnapshot = ref('')

const { data: known, refresh } = await useAsyncData(
  'admin-locales',
  () => auth.request<LocaleInfo[]>('/api/launcher/locales'),
  { default: () => [] as LocaleInfo[] }
)

const rows = computed(() => {
  const needle = search.value.trim().toLowerCase()
  return [...builtin.value.entries()]
    .map(([key, base]) => ({ key, base, override: overrides.get(key) || '' }))
    .filter(r => !onlyChanged.value || r.override)
    .filter(r =>
      !needle
      || r.key.toLowerCase().includes(needle)
      || r.base.toLowerCase().includes(needle)
      || r.override.toLowerCase().includes(needle))
})

const changedCount = computed(() => [...overrides.values()].filter(Boolean).length)
const currentFtl = computed(() =>
  buildFtl([...overrides.entries()].map(([key, value]) => ({ key, value })))
)
const dirty = computed(() => currentFtl.value !== savedSnapshot.value)

async function load(code: string) {
  active.value = code
  message.value = null
  error.value = null
  try {
    const catalog = await auth.request<Catalog>(`/api/launcher/locales/${code}`)
    builtin.value = new Map(parseFtl(catalog.builtin).map(e => [e.key, e.value]))
    overrides.clear()
    for (const entry of parseFtl(catalog.ftl)) overrides.set(entry.key, entry.value)
    savedSnapshot.value = currentFtl.value
  } catch (err) {
    error.value = 'Could not load the catalog.'
    console.error(err)
  }
}

function setOverride(key: string, value: string) {
  if (value.trim()) overrides.set(key, value)
  else overrides.delete(key)
}

async function save() {
  busy.value = true
  message.value = null
  error.value = null
  try {
    await auth.request(`/api/admin/locales/${active.value}`, {
      method: 'PUT',
      body: { ftl: currentFtl.value }
    })
    savedSnapshot.value = currentFtl.value
    message.value = 'Saved. Launchers pick it up over WebSocket.'
    await refresh()
  } catch (err) {
    error.value = 'Save failed.'
    console.error(err)
  } finally {
    busy.value = false
  }
}

async function resetAll() {
  busy.value = true
  error.value = null
  try {
    await auth.request(`/api/admin/locales/${active.value}`, { method: 'DELETE' })
    overrides.clear()
    savedSnapshot.value = currentFtl.value
    message.value = 'All overrides removed.'
    await refresh()
  } catch (err) {
    error.value = 'Could not remove overrides.'
    console.error(err)
  } finally {
    busy.value = false
  }
}

await load(active.value)
</script>

<template>
  <NoroShell :title="t('admin-i18n-title')" :subtitle="t('admin-i18n-subtitle')">
    <div class="grid gap-4">
      <section class="noro-panel p-6">
        <div class="flex flex-wrap items-center gap-2">
          <AtomButton
            v-for="loc in LOCALES"
            :key="loc.code"
            :variant="active === loc.code ? 'primary' : 'secondary'"
            size="sm"
            equal
            @click="load(loc.code)"
          >
            {{ loc.label }}
          </AtomButton>
          <span class="noro-label ml-auto">
            {{ t('admin-i18n-changed-count', { count: changedCount, total: builtin.size }) }}
          </span>
        </div>

        <p class="mt-4 text-sm leading-6 text-[var(--noro-muted)]">
          {{ t('admin-i18n-hint') }}
        </p>

        <div class="mt-4 flex flex-wrap items-center gap-3">
          <input v-model="search" class="noro-input max-w-xs" :placeholder="t('admin-i18n-search-placeholder')">
          <label class="flex cursor-pointer items-center gap-2 text-sm text-[var(--noro-muted)]">
            <input v-model="onlyChanged" type="checkbox" class="size-4">
            {{ t('admin-i18n-only-changed') }}
          </label>
        </div>
      </section>

      <section class="noro-panel overflow-hidden">
        <div class="max-h-[60vh] overflow-auto noro-scroll">
          <table class="noro-table">
            <thead class="sticky top-0 z-10">
              <tr>
                <th class="w-[220px]">{{ t('admin-i18n-col-key') }}</th>
                <th class="w-[38%]">{{ t('admin-i18n-col-builtin') }}</th>
                <th>{{ t('admin-i18n-col-override') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in rows" :key="row.key">
                <td class="align-top">
                  <code class="text-xs text-[var(--noro-blue)]">{{ row.key }}</code>
                </td>
                <td class="align-top">
                  <pre class="whitespace-pre-wrap break-words text-xs leading-5 text-[var(--noro-muted)]">{{ row.base }}</pre>
                </td>
                <td class="align-top">
                  <textarea
                    v-if="isMultiline(row.base) || isMultiline(row.override)"
                    :value="row.override"
                    rows="4"
                    spellcheck="false"
                    class="noro-input w-full font-mono text-xs leading-5"
                    :placeholder="row.base"
                    @input="setOverride(row.key, ($event.target as HTMLTextAreaElement).value)"
                  />
                  <input
                    v-else
                    :value="row.override"
                    class="noro-input w-full text-sm"
                    :placeholder="row.base"
                    @input="setOverride(row.key, ($event.target as HTMLInputElement).value)"
                  >
                </td>
              </tr>
              <tr v-if="!rows.length">
                <td colspan="3" class="py-10 text-center text-sm text-[var(--noro-muted)]">
                  {{ t('admin-i18n-no-match') }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section v-if="can('noro.admin.translations.edit')" class="noro-panel flex flex-wrap items-center gap-3 p-4">
        <AtomButton
          variant="primary"
          icon="i-lucide-save"
          :disabled="busy || !dirty"
          @click="save"
        >
          {{ busy ? t('web-rules-save') : t('cabinet-save') }}
        </AtomButton>
        <span v-if="dirty" class="noro-label text-[var(--noro-amber)]">{{ t('admin-i18n-unsaved') }}</span>
        <AtomButton
          v-if="changedCount"
          variant="ghost"
          icon="i-lucide-rotate-ccw"
          :disabled="busy"
          @click="resetAll"
          class="ml-auto text-[var(--noro-danger)]"
        >
          {{ t('admin-i18n-reset-all') }}
        </AtomButton>
      </section>

      <UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-circle-alert" :description="error" />
      <UAlert v-else-if="message" color="success" variant="subtle" icon="i-lucide-check" :description="message" />
    </div>
  </NoroShell>
</template>
