<script setup lang="ts">
/**
 * Правка правила. Наказания заданы вилками, а не одним числом: правило
 * описывает, что вообще допустимо, а срок в этих рамках выбирает модератор.
 */
import type { Rule, RuleCategory, RuleSanction } from '~/types/rules'
import type { ServerRow } from '~/types/api'
import type { SanctionDraft } from '~/components/admin/SanctionEditor.vue'

const props = defineProps<{
  draft: Partial<Rule>
  sanctions: RuleSanction[]
  categories: RuleCategory[]
  servers: ServerRow[]
}>()
const emit = defineEmits<{ save: [Record<string, unknown>] }>()
const open = defineModel<boolean>({ required: true })
const { t } = useT()

/** В форме пустая строка вместо `null`: `<select>` хранит только строки. */
const form = reactive({ id: '', code: '', category_id: '', server_id: '' })
const sanctions = ref<SanctionDraft[]>([])
const { items: localized, base, load: loadText, payload: translations } = useLocalizedDraft()

// Модалка живёт в дереве постоянно, поэтому форму пересобираем на каждый
// черновик — иначе второе открытие показало бы поля предыдущего правила.
watch(() => props.draft, (draft) => {
  Object.assign(form, {
    id: draft.id || '',
    code: draft.code || '',
    category_id: draft.category_id || '',
    server_id: draft.server_id || '',
  })
  sanctions.value = props.sanctions.map(s => ({
    kind: s.kind,
    label: s.label,
    min: s.min_minutes ? formatDuration(s.min_minutes) : '',
    max: s.max_minutes ? formatDuration(s.max_minutes) : '',
  }))
  loadText(
    { title: draft.title || '', description: draft.description || '', punish_reason: draft.punish_reason || '' },
    draft.id ? `/api/admin/rules/${draft.id}/translations` : undefined,
  )
}, { immediate: true })

const broken = computed(() => sanctions.value.some(s => [s.min, s.max].some(
  value => value.trim() && Number.isNaN(parseDuration(value)),
)))
const valid = computed(() => !!form.code.trim() && !!base.value.title.trim() && !broken.value)

function submit() {
  emit('save', {
    id: form.id || undefined,
    code: form.code.trim(),
    title: base.value.title.trim(),
    description: base.value.description.trim(),
    punish_reason: (base.value.punish_reason ?? '').trim(),
    category_id: form.category_id || null,
    server_id: form.server_id || null,
    sanctions: sanctions.value.map(s => ({
      kind: s.kind,
      label: s.label.trim(),
      min_minutes: s.kind === 'warn' ? null : parseDuration(s.min),
      max_minutes: s.kind === 'warn' ? null : parseDuration(s.max),
    })),
    translations: translations(),
  })
}
</script>

<template>
  <NoroModal v-model="open" :title="form.id ? t('admin-rule-edit', { code: form.code }) : t('admin-rule-new')">
    <div class="grid gap-4">
      <div class="grid gap-4 sm:grid-cols-[128px_1fr_1fr]">
        <label class="block">
          <span class="noro-label">{{ t('admin-rule-code') }}</span>
          <input v-model="form.code" class="noro-input w-full font-bold" placeholder="1.1">
        </label>
        <label class="block">
          <span class="noro-label">{{ t('admin-rule-section') }}</span>
          <NoroSelect v-model="form.category_id" class="w-full">
            <option value="">{{ t('admin-rule-no-section') }}</option>
            <option v-for="c in categories" :key="c.id" :value="c.id">
              {{ c.code ? `${c.code} · ` : '' }}{{ c.name }}
            </option>
          </NoroSelect>
        </label>
        <label class="block">
          <span class="noro-label">{{ t('web-rules-scope') }}</span>
          <NoroSelect v-model="form.server_id" class="w-full">
            <option value="">{{ t('web-rules-scope-general') }}</option>
            <option v-for="s in servers" :key="s.id" :value="s.id">{{ t('admin-rule-server-only', { name: s.name }) }}</option>
          </NoroSelect>
        </label>
      </div>

      <AdminLocalizedText
        v-model="localized"
        :title-label="t('web-rules-title')"
        :title-placeholder="t('admin-rule-title-placeholder')"
        :text-label="t('admin-rule-wording')"
        :text-placeholder="t('admin-rule-text-placeholder')"
        :reason-label="t('admin-rule-punish-reason')"
        :reason-placeholder="t('admin-rule-punish-reason-placeholder')"
        :reason-hint="t('admin-rule-punish-reason-hint')"
      />

      <AdminSanctionEditor v-model="sanctions" />

      <div class="flex justify-end gap-2">
        <AtomButton variant="secondary" @click="open = false">{{ t('web-rules-cancel') }}</AtomButton>
        <AtomButton variant="primary" :disabled="!valid" @click="submit">
          {{ form.id ? t('web-rules-save') : t('admin-rule-create') }}
        </AtomButton>
      </div>
    </div>
  </NoroModal>
</template>
