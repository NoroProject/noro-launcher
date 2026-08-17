<script setup lang="ts">
/** Правка раздела свода. */
import type { RuleCategory } from '~/types/rules'
import type { ServerRow } from '~/types/api'

const props = defineProps<{
  draft: Partial<RuleCategory>
  categories: RuleCategory[]
  servers: ServerRow[]
}>()
const emit = defineEmits<{ save: [Record<string, unknown>] }>()
const open = defineModel<boolean>({ required: true })
const { t } = useT()

const form = reactive({ id: '', code: '', parent_id: '', server_id: '' })
const { items: localized, base, load: loadText, payload: translations } = useLocalizedDraft()

watch(() => props.draft, (draft) => {
  Object.assign(form, {
    id: draft.id || '',
    code: draft.code || '',
    parent_id: draft.parent_id || '',
    server_id: draft.server_id || '',
  })
  loadText(
    { title: draft.name || '', description: draft.description || '' },
    draft.id ? `/api/admin/rules/categories/${draft.id}/translations` : undefined,
  )
}, { immediate: true })

/** Раздел не может быть вложен сам в себя — этот вариант просто не предлагаем. */
const parents = computed(() => props.categories.filter(c => c.id !== form.id))

function submit() {
  emit('save', {
    id: form.id || undefined,
    code: form.code.trim(),
    name: base.value.title.trim(),
    description: base.value.description.trim(),
    parent_id: form.parent_id || null,
    server_id: form.server_id || null,
    translations: translations(),
  })
}
</script>

<template>
  <NoroModal v-model="open" :title="form.id ? t('admin-cat-edit', { name: base.title }) : t('admin-cat-new')">
    <div class="grid gap-4">
      <div class="grid gap-4 sm:grid-cols-[128px_1fr_1fr]">
        <label class="block">
          <span class="noro-label">{{ t('admin-cat-number') }}</span>
          <input v-model="form.code" class="noro-input w-full font-bold" placeholder="1">
        </label>
        <label class="block">
          <span class="noro-label">{{ t('admin-cat-parent') }}</span>
          <NoroSelect v-model="form.parent_id" class="w-full">
            <option value="">{{ t('admin-cat-top') }}</option>
            <option v-for="c in parents" :key="c.id" :value="c.id">
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
        :title-label="t('admin-rule-code')"
        title-placeholder="Gameplay"
        :text-label="t('admin-cat-intro')"
        text-placeholder="One or two sentences shown above the rules of this section"
      />

      <div class="flex justify-end gap-2">
        <AtomButton variant="secondary" @click="open = false">{{ t('web-rules-cancel') }}</AtomButton>
        <AtomButton variant="primary" :disabled="!base.title.trim()" @click="submit">
          {{ form.id ? t('web-rules-save') : t('admin-cat-create') }}
        </AtomButton>
      </div>
    </div>
  </NoroModal>
</template>
