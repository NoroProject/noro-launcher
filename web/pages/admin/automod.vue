<script setup lang="ts">
interface ChatFilterItem {
  filter_type: string
  mode: string
  enabled: boolean
  rule_code: string | null
  whitelist: string[]
  words: string[]
  threshold: number
  min_length: number
  max_messages: number
  window_secs: number
}

const auth = useAuth()
const { t } = useT()
const notify = useNotify()
await auth.loadMe()

const filters = ref<ChatFilterItem[]>([])
const pending = ref(false)
const savingFilter = ref<string | null>(null)

// Draft state per filter_type
const drafts = reactive<Record<string, {
  mode: string
  enabled: boolean
  rule_code: string
  whitelistRaw: string
  wordsRaw: string
  threshold: number
  min_length: number
  max_messages: number
  window_secs: number
}>>({})

async function load() {
  pending.value = true
  try {
    const list = await auth.request<ChatFilterItem[]>('/api/admin/chat-filters')
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
    notify.fail(e, 'Failed to load chat filters')
  } finally {
    pending.value = false
  }
}

async function saveFilter(filterType: string) {
  const d = drafts[filterType]
  if (!d) return
  savingFilter.value = filterType
  try {
    const payload: ChatFilterItem = {
      filter_type: filterType,
      mode: d.mode,
      enabled: d.enabled,
      rule_code: d.rule_code.trim() || null,
      whitelist: d.whitelistRaw ? d.whitelistRaw.split(',').map(s => s.trim()).filter(Boolean) : [],
      words: d.wordsRaw ? d.wordsRaw.split(',').map(s => s.trim()).filter(Boolean) : [],
      threshold: Number(d.threshold) || 0,
      min_length: Number(d.min_length) || 0,
      max_messages: Number(d.max_messages) || 0,
      window_secs: Number(d.window_secs) || 0,
    }
    await auth.request('/api/admin/chat-filters', {
      method: 'PUT',
      body: payload,
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
  <NoroShell title="AutoMod Chat Filters" subtitle="Configure automated chat moderation rules and thresholds">
    <template #actions>
      <AtomButton icon="i-lucide-refresh-cw" variant="dark" :loading="pending" @click="load()">
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
    </template>

    <div v-if="filters.length" class="grid gap-6">
      <div
        v-for="item in filters"
        :key="item.filter_type"
        class="noro-panel p-6 grid gap-4"
      >
        <div class="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--noro-border)] pb-4">
          <div class="flex items-center gap-3">
            <UIcon
              :name="item.filter_type === 'ad' ? 'i-lucide-megaphone' : item.filter_type === 'word' ? 'i-lucide-shield-alert' : item.filter_type === 'caps' ? 'i-lucide-type' : 'i-lucide-waves'"
              class="size-6 text-[var(--noro-blue)]"
            />
            <div>
              <h3 class="text-base font-bold uppercase text-white">{{ item.filter_type }} Filter</h3>
              <p class="text-xs text-[var(--noro-muted)]">
                {{ item.filter_type === 'ad' ? 'Detect domain links, URLs, and IP addresses' : item.filter_type === 'word' ? 'Block banned words and profanity' : item.filter_type === 'caps' ? 'Limit excessive CAPITAL LETTERS' : 'Prevent message flooding and rapid spam' }}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-4">
            <label class="flex items-center gap-2 cursor-pointer select-none text-sm font-bold">
              <input v-if="drafts[item.filter_type]" v-model="drafts[item.filter_type].enabled" type="checkbox" class="accent-[var(--noro-magenta)]">
              <span :class="drafts[item.filter_type]?.enabled ? 'text-emerald-400' : 'text-red-400'">
                {{ drafts[item.filter_type]?.enabled ? 'ENABLED' : 'DISABLED' }}
              </span>
            </label>
            <AtomButton
              icon="i-lucide-save"
              variant="primary"
              :loading="savingFilter === item.filter_type"
              @click="saveFilter(item.filter_type)"
            >
              {{ t('cabinet-save') }}
            </AtomButton>
          </div>
        </div>

        <div v-if="drafts[item.filter_type]" class="grid gap-4 md:grid-cols-3">
          <label class="block">
            <span class="noro-label mb-1.5 block">Action Mode</span>
            <NoroSelect v-model="drafts[item.filter_type].mode" class="w-full">
              <option value="deny">DENY (Block Message)</option>
              <option value="escalate">ESCALATE (Warn then Auto-Mute)</option>
              <option value="punish">PUNISH (Immediate Auto-Mute)</option>
              <option value="shadow">SHADOW (Allow + Notify Staff)</option>
            </NoroSelect>
          </label>

          <label class="block">
            <span class="noro-label mb-1.5 block">Rule Code (Catalog Link)</span>
            <input
              v-model="drafts[item.filter_type].rule_code"
              class="noro-input w-full"
              placeholder="e.g. 1.4 or 2.2"
            >
          </label>

          <!-- Specific Fields for Ad Filter -->
          <template v-if="item.filter_type === 'ad'">
            <label class="block md:col-span-3">
              <span class="noro-label mb-1.5 block">Allowed Domains Whitelist (comma-separated)</span>
              <input
                v-model="drafts[item.filter_type].whitelistRaw"
                class="noro-input w-full"
                placeholder="noro.dalynkaa.dev, dalynkaa.dev"
              >
            </label>
          </template>

          <!-- Specific Fields for Word Filter -->
          <template v-if="item.filter_type === 'word'">
            <label class="block md:col-span-3">
              <span class="noro-label mb-1.5 block">Forbidden Words List (comma-separated)</span>
              <input
                v-model="drafts[item.filter_type].wordsRaw"
                class="noro-input w-full"
                placeholder="badword1, badword2"
              >
            </label>
          </template>

          <!-- Specific Fields for Caps Filter -->
          <template v-if="item.filter_type === 'caps'">
            <label class="block">
              <span class="noro-label mb-1.5 block">Caps Threshold Ratio (0.1 - 1.0)</span>
              <input
                v-model.number="drafts[item.filter_type].threshold"
                type="number"
                step="0.05"
                min="0.1"
                max="1.0"
                class="noro-input w-full"
              >
            </label>
            <label class="block">
              <span class="noro-label mb-1.5 block">Minimum Message Length</span>
              <input
                v-model.number="drafts[item.filter_type].min_length"
                type="number"
                min="1"
                class="noro-input w-full"
              >
            </label>
          </template>

          <!-- Specific Fields for Flood / Escalate Filters -->
          <template v-if="item.filter_type === 'flood' || drafts[item.filter_type].mode === 'escalate'">
            <label class="block">
              <span class="noro-label mb-1.5 block">Max Messages Limit</span>
              <input
                v-model.number="drafts[item.filter_type].max_messages"
                type="number"
                min="1"
                class="noro-input w-full"
              >
            </label>
            <label class="block">
              <span class="noro-label mb-1.5 block">Time Window (seconds)</span>
              <input
                v-model.number="drafts[item.filter_type].window_secs"
                type="number"
                min="1"
                class="noro-input w-full"
              >
            </label>
          </template>
        </div>
      </div>
    </div>

    <EmptyState
      v-else-if="!pending"
      icon="i-lucide-shield-alert"
      title="No AutoMod Filters"
      text="AutoMod chat filters are currently empty."
    />
  </NoroShell>
</template>
