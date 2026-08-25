<script setup lang="ts">
/** Тексты, которые игрок видит при бане, муте и предупреждении. */
import { MODERATION_MESSAGE_FIELDS, type ModerationMessages } from '~/types/moderation'

const auth = useAuth()
const { t } = useT()
const notify = useNotify()
await auth.loadMe()

const canEdit = computed(() => auth.hasPermission('noro.admin.moderation.edit') || auth.hasPermission('noro.admin.settings.edit'))
const activeLang = ref<string>('ru')
const map = ref<Record<string, ModerationMessages>>({})
const defaultsMap = ref<Record<string, ModerationMessages>>({})
const pending = ref(false)

const { data: remoteLocales } = await useAsyncData('moderation-locales', async () => {
  try {
    const list = await auth.request<{ locale: string }[]>('/api/launcher/locales')
    if (list && list.length) {
      return list.map(l => l.locale)
    }
  } catch {
    // fallback
  }
  return ['ru', 'en']
})

const availableLangs = computed(() => {
  const keys = new Set([...(remoteLocales.value || []), ...Object.keys(map.value), 'ru', 'en'])
  return Array.from(keys).map(code => getLocaleOption(code))
})

const currentDraft = computed(() => {
  if (!map.value[activeLang.value]) {
    map.value[activeLang.value] = { ...defaultsMap.value.en || defaultsMap.value.ru || {} as ModerationMessages }
  }
  return map.value[activeLang.value]
})

async function load() {
  pending.value = true
  try {
    const data = await auth.request<{
      messages: ModerationMessages
      map?: Record<string, ModerationMessages>
      defaults: ModerationMessages
      defaults_map?: Record<string, ModerationMessages>
    }>('/api/admin/moderation/messages')

    map.value = {}
    if (data.map && Object.keys(data.map).length) {
      for (const [k, v] of Object.entries(data.map)) {
        map.value[k] = { ...v }
      }
    } else {
      map.value = { ru: { ...data.messages }, en: { ...data.messages } }
    }

    defaultsMap.value = {
      ru: data.defaults_map?.ru ? { ...data.defaults_map.ru } : { ...data.defaults },
      en: data.defaults_map?.en ? { ...data.defaults_map.en } : { ...data.defaults },
    }
  } catch (e) {
    notify.fail(e, 'Failed to load moderation messages')
  } finally {
    pending.value = false
  }
}

async function save() {
  pending.value = true
  try {
    await auth.request('/api/admin/moderation/messages', {
      method: 'PUT',
      body: map.value,
    })
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  } finally {
    pending.value = false
  }
}

function reset(key: keyof ModerationMessages) {
  const def = defaultsMap.value[activeLang.value] || defaultsMap.value.en
  if (map.value[activeLang.value] && def) {
    map.value[activeLang.value][key] = def[key]
  }
}

onMounted(() => load())
</script>

<template>
  <NoroShell :title="t('admin-moderation-title')" :subtitle="t('admin-moderation-subtitle')">
    <template #actions>
      <AtomButton v-if="canEdit" icon="i-lucide-save" :loading="pending" @click="save">
        {{ t('cabinet-save') }}
      </AtomButton>
    </template>

    <div class="grid gap-4 xl:grid-cols-[1fr_320px]">
      <section class="noro-panel grid gap-4 p-6">
        <!-- Вкладки языков над инпутами -->
        <div class="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--noro-border)] pb-4">
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-xs font-bold text-[var(--noro-muted)] uppercase tracking-wider">Язык / Language:</span>
            <div class="flex flex-wrap items-center gap-1 bg-[var(--noro-input)] p-1 rounded-lg border border-[var(--noro-border)]">
              <button
                v-for="loc in availableLangs"
                :key="loc.code"
                type="button"
                class="group relative flex items-center gap-1.5 px-3 py-1 text-xs font-bold rounded-md transition cursor-pointer"
                :class="activeLang === loc.code ? 'bg-[var(--noro-cream)] text-[var(--noro-on-cream)] shadow' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
                @click="activeLang = loc.code"
              >
                <span>{{ loc.label }} ({{ loc.short }})</span>
              </button>
            </div>
          </div>
        </div>

        <div v-for="field in MODERATION_MESSAGE_FIELDS" :key="field.key" class="grid gap-2">
          <div class="flex items-center justify-between gap-4">
            <label class="text-sm font-bold text-[var(--noro-text)]">
              {{ t(field.label) }}
            </label>
            <button
              v-if="canEdit"
              type="button"
              class="text-xs text-[var(--noro-muted)] hover:text-[var(--noro-cream)]"
              @click="reset(field.key)"
            >
              {{ t('admin-moderation-reset') }}
            </button>
          </div>
          <p class="text-xs text-[var(--noro-muted)]">{{ t(field.hint) }}</p>
          <NoroTextarea
            v-if="currentDraft"
            v-model="currentDraft[field.key]"
            :rows="field.rows"
            :disabled="!canEdit"
          />
        </div>
      </section>

      <aside class="noro-panel h-fit p-6 space-y-4">
        <div>
          <h2 class="mb-2 text-sm font-bold uppercase text-[var(--noro-cream)]">
            {{ t('admin-moderation-vars-title') }}
          </h2>
          <p class="mb-4 text-xs text-[var(--noro-muted)]">{{ t('admin-moderation-vars-lead') }}</p>
          <ul class="grid gap-2 text-xs">
            <li
              v-for="name in ['player', 'reason', 'duration', 'expires', 'actor', 'rule', 'rule-title', 'rule-link', 'id', 'kind']"
              :key="name"
            >
              <code class="text-[var(--noro-blue)]">{{ '{' + name.replace('-', '_') + '}' }}</code>
              <span class="ml-2 text-[var(--noro-muted)]">{{ t(`admin-moderation-var-${name}`) }}</span>
            </li>
          </ul>
        </div>
      </aside>
    </div>
  </NoroShell>
</template>
