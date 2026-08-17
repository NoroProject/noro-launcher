<script setup lang="ts">
/** Тексты, которые игрок видит при бане, муте и предупреждении. */
import { MODERATION_MESSAGE_FIELDS, type ModerationMessages } from '~/types/moderation'

const auth = useAuth()
const { t } = useT()
const notify = useNotify()
await auth.loadMe()

const canEdit = computed(() => auth.hasPermission('noro.admin.settings.edit'))
const draft = ref<ModerationMessages | null>(null)
const defaults = ref<ModerationMessages | null>(null)
const pending = ref(false)

async function load() {
  pending.value = true
  try {
    const data = await auth.request<{ messages: ModerationMessages; defaults: ModerationMessages }>(
      '/api/admin/moderation/messages',
    )
    draft.value = { ...data.messages }
    defaults.value = data.defaults
  } catch (e) {
    notify.fail(e, 'Failed to load moderation messages')
  } finally {
    pending.value = false
  }
}

async function save() {
  if (!draft.value) return
  pending.value = true
  try {
    await auth.request('/api/admin/moderation/messages', { method: 'PUT', body: draft.value })
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  } finally {
    pending.value = false
  }
}

/** Вернуть один текст к встроенному: правится по одному, сбрасывается тоже. */
function reset(key: keyof ModerationMessages) {
  if (draft.value && defaults.value) draft.value[key] = defaults.value[key]
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
          <UTextarea
            v-if="draft"
            v-model="draft[field.key]"
            :rows="field.rows"
            :disabled="!canEdit"
            autoresize
            class="font-mono text-xs"
          />
        </div>
      </section>

      <aside class="noro-panel h-fit p-6">
        <h2 class="mb-2 text-sm font-bold uppercase text-[var(--noro-cream)]">
          {{ t('admin-moderation-vars-title') }}
        </h2>
        <p class="mb-4 text-xs text-[var(--noro-muted)]">{{ t('admin-moderation-vars-lead') }}</p>
        <ul class="grid gap-2 text-xs">
          <li v-for="name in ['player', 'reason', 'duration', 'expires', 'actor', 'rule', 'id', 'kind']" :key="name">
            <code class="text-[var(--noro-blue)]">{{ '{' + name + '}' }}</code>
            <span class="ml-2 text-[var(--noro-muted)]">{{ t(`admin-moderation-var-${name}`) }}</span>
          </li>
        </ul>
      </aside>
    </div>
  </NoroShell>
</template>
