<script setup lang="ts">
/** Автомодерация чата: четыре фильтра, каждый со своим режимом и порогами. */
import type { FilterDraft } from '~/components/admin/AutomodFilter.vue'

interface ChatFilterItem extends Omit<FilterDraft, 'rule_code' | 'whitelistRaw' | 'wordsRaw'> {
  filter_type: string
  rule_code: string | null
  whitelist: string[]
  words: string[]
}

const auth = useAuth()
const { t } = useT()
const notify = useNotify()
await auth.loadMe()

const filters = ref<ChatFilterItem[]>([])
const pending = ref(false)
const savingFilter = ref<string | null>(null)

const drafts = reactive<Record<string, FilterDraft>>({})

/** Списки правятся строкой через запятую: так их и вводят. */
function split(raw: string) {
  return raw ? raw.split(',').map(s => s.trim()).filter(Boolean) : []
}

async function load() {
  pending.value = true
  try {
    const list = await auth.requestList<ChatFilterItem>('/api/admin/chat-filters')
    filters.value = list
    for (const item of list) {
      drafts[item.filter_type] = {
        mode: item.mode || 'deny',
        enabled: item.enabled ?? true,
        rule_code: item.rule_code || '',
        whitelistRaw: (item.whitelist || []).join(', '),
        wordsRaw: (item.words || []).join(', '),
        threshold: item.threshold || 0.6,
        min_length: item.min_length || 6,
        max_messages: item.max_messages || 3,
        window_secs: item.window_secs || 4,
      }
    }
  } catch (e) {
    notify.fail(e, t('admin-automod-load-failed'))
  } finally {
    pending.value = false
  }
}

async function saveFilter(filterType: string) {
  const d = drafts[filterType]
  if (!d) return
  savingFilter.value = filterType
  try {
    await auth.request('/api/admin/chat-filters', {
      method: 'PUT',
      body: {
        filter_type: filterType,
        mode: d.mode,
        enabled: d.enabled,
        rule_code: d.rule_code.trim() || null,
        whitelist: split(d.whitelistRaw),
        words: split(d.wordsRaw),
        threshold: Number(d.threshold) || 0,
        min_length: Number(d.min_length) || 0,
        max_messages: Number(d.max_messages) || 0,
        window_secs: Number(d.window_secs) || 0,
      } satisfies ChatFilterItem,
    })
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  } finally {
    savingFilter.value = null
  }
}

onMounted(() => load())
</script>

<template>
  <NoroShell :title="t('admin-automod-title')" :subtitle="t('admin-automod-subtitle')">
    <template #actions>
      <AtomButton icon="i-lucide-refresh-cw" variant="dark" :loading="pending" @click="load()">
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
    </template>

    <div v-if="filters.length" class="grid gap-6">
      <AdminAutomodFilter
        v-for="item in filters"
        :key="item.filter_type"
        v-model="drafts[item.filter_type]"
        :filter-type="item.filter_type"
        :saving="savingFilter === item.filter_type"
        @save="saveFilter(item.filter_type)"
      />
    </div>

    <EmptyState
      v-else-if="!pending"
      icon="i-lucide-shield-alert"
      :title="t('admin-automod-empty-title')"
      :text="t('admin-automod-empty-text')"
    />
  </NoroShell>
</template>
