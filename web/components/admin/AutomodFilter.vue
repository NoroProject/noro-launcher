<script setup lang="ts">
/**
 * Карточка одного фильтра чата: режим, привязка к пункту свода и пороги.
 *
 * Поля показываются по виду фильтра: таблица `chat_filters` одна на все четыре,
 * и `threshold` в строке рекламного фильтра не значит ничего.
 */
export interface FilterDraft {
  mode: string
  enabled: boolean
  rule_code: string
  whitelistRaw: string
  wordsRaw: string
  threshold: number
  min_length: number
  max_messages: number
  window_secs: number
}

const props = withDefaults(defineProps<{ filterType: string, saving: boolean, canEdit?: boolean }>(), { canEdit: true })
defineEmits<{ save: [] }>()
const draft = defineModel<FilterDraft>({ required: true })
const { t } = useT()

/** Виды, которые знает свод. Незнакомый приезжает из базы под своим именем. */
const KNOWN: Record<string, string> = {
  ad: 'i-lucide-megaphone',
  word: 'i-lucide-shield-alert',
  caps: 'i-lucide-type',
  flood: 'i-lucide-waves',
}

const known = computed(() => props.filterType in KNOWN)
const icon = computed(() => KNOWN[props.filterType] || 'i-lucide-filter')
const title = computed(() => known.value ? t(`admin-automod-${props.filterType}-title`) : props.filterType)
const hint = computed(() => known.value ? t(`admin-automod-${props.filterType}-hint`) : '')

const MODES = ['deny', 'escalate', 'punish', 'shadow']

// Порог срабатывания нужен и флуд-фильтру, и любому другому в режиме
// `escalate`: там наказание выдаётся не сразу, а после серии сообщений.
const counted = computed(() => props.filterType === 'flood' || draft.value.mode === 'escalate')
</script>

<template>
  <div class="noro-panel grid gap-4 p-6">
    <div class="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--noro-border)] pb-4">
      <div class="flex items-center gap-3">
        <UIcon :name="icon" class="size-6 text-[var(--noro-blue)]" />
        <div>
          <h3 class="text-base font-bold uppercase text-white">{{ title }}</h3>
          <p class="text-xs text-[var(--noro-muted)]">{{ hint }}</p>
        </div>
      </div>
      <div class="flex items-center gap-4">
        <label class="flex cursor-pointer select-none items-center gap-2 text-sm font-bold">
          <input v-model="draft.enabled" type="checkbox" class="accent-[var(--noro-magenta)]">
          <span :class="draft.enabled ? 'text-emerald-400' : 'text-red-400'">
            {{ draft.enabled ? t('admin-automod-enabled') : t('admin-automod-disabled') }}
          </span>
        </label>
        <AtomButton v-if="canEdit" icon="i-lucide-save" variant="primary" :loading="saving" @click="$emit('save')">
          {{ t('cabinet-save') }}
        </AtomButton>
      </div>
    </div>

    <div class="grid gap-4 md:grid-cols-3">
      <label class="block">
        <span class="noro-label mb-1.5 block">{{ t('admin-automod-mode') }}</span>
        <NoroSelect v-model="draft.mode" class="w-full">
          <option v-for="mode in MODES" :key="mode" :value="mode">{{ t(`admin-automod-mode-${mode}`) }}</option>
        </NoroSelect>
      </label>

      <label class="block">
        <span class="noro-label mb-1.5 block">{{ t('admin-automod-rule-code') }}</span>
        <input
          v-model="draft.rule_code"
          class="noro-input w-full"
          :placeholder="t('admin-automod-rule-code-hint')"
        >
      </label>

      <label v-if="filterType === 'ad'" class="block md:col-span-3">
        <span class="noro-label mb-1.5 block">{{ t('admin-automod-whitelist') }}</span>
        <input v-model="draft.whitelistRaw" class="noro-input w-full" placeholder="example.com, cdn.example.com">
      </label>

      <label v-if="filterType === 'word'" class="block md:col-span-3">
        <span class="noro-label mb-1.5 block">{{ t('admin-automod-words') }}</span>
        <input v-model="draft.wordsRaw" class="noro-input w-full" :placeholder="t('admin-automod-words-hint')">
      </label>

      <template v-if="filterType === 'caps'">
        <label class="block">
          <span class="noro-label mb-1.5 block">{{ t('admin-automod-threshold') }}</span>
          <input v-model.number="draft.threshold" type="number" step="0.05" min="0.1" max="1.0" class="noro-input w-full">
        </label>
        <label class="block">
          <span class="noro-label mb-1.5 block">{{ t('admin-automod-min-length') }}</span>
          <input v-model.number="draft.min_length" type="number" min="1" class="noro-input w-full">
        </label>
      </template>

      <template v-if="counted">
        <label class="block">
          <span class="noro-label mb-1.5 block">{{ t('admin-automod-max-messages') }}</span>
          <input v-model.number="draft.max_messages" type="number" min="1" class="noro-input w-full">
        </label>
        <label class="block">
          <span class="noro-label mb-1.5 block">{{ t('admin-automod-window') }}</span>
          <input v-model.number="draft.window_secs" type="number" min="1" class="noro-input w-full">
        </label>
      </template>
    </div>
  </div>
</template>
