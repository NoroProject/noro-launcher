<script setup lang="ts">
/** Свод правил: разделы, правила, порядок. */
import type { Rule, RuleActions, RuleCategory } from '~/types/rules'
import { RULE_ACTIONS } from '~/types/rules'

const store = useAdminRules()
const auth = useAuth()
const { t } = useT()
/** Дерево правок читается и без права на правку: свод полезен и так. */
const canEdit = computed(() => auth.hasPermission('noro.admin.rules.edit'))
const canDelete = computed(() => auth.hasPermission('noro.admin.rules.delete'))
const { tree, loose, categories, rules, servers, pending, scope, search } = store

const showRule = ref(false)
const showCategory = ref(false)
const ruleDraft = ref<Partial<Rule>>({})
const categoryDraft = ref<Partial<RuleCategory>>({})

onMounted(store.load)

const actions: RuleActions = {
  editRule(rule) {
    ruleDraft.value = { ...rule }
    showRule.value = true
  },
  /** Новое правило дописывается в конец раздела: код и свод — оттуда же. */
  newRule(category) {
    const siblings = rules.value.filter(r => r.category_id === (category?.id ?? null))
    ruleDraft.value = {
      category_id: category?.id ?? null,
      server_id: category?.server_id ?? (scope.value || null),
      code: nextRuleCode(category?.code ?? '', siblings),
    }
    showRule.value = true
  },
  editCategory(category) {
    categoryDraft.value = { ...category }
    showCategory.value = true
  },
  newSection(parent) {
    const siblings = categories.value.filter(c => c.parent_id === (parent?.id ?? null))
    categoryDraft.value = {
      parent_id: parent?.id ?? null,
      server_id: parent?.server_id ?? (scope.value || null),
      code: nextCategoryCode(parent?.code ?? '', siblings),
    }
    showCategory.value = true
  },
  async removeRule(rule) {
    if (!confirm(t('admin-rule-delete-confirm', { code: rule.code, title: rule.title }))) return
    try {
      await store.remove('/api/admin/rules', rule.id)
    } catch (e) {
      store.notify.fail(e, t('admin-rule-delete-fail'))
    }
  },
  async removeCategory(category) {
    if (!confirm(t('admin-cat-delete-confirm', { name: category.name }))) return
    try {
      await store.remove('/api/admin/rules/categories', category.id)
    } catch (e) {
      store.notify.fail(e, t('admin-cat-delete-fail'))
    }
  },
  move: store.move,
  serverName: store.serverName,
  sanctionsOf: store.sanctionsOf,
  canEdit: canEdit.value,
  canDelete: canDelete.value,
}
provide(RULE_ACTIONS, actions)

async function saveRule(body: Record<string, unknown>) {
  try {
    await store.save('/api/admin/rules', body, body.id as string | undefined)
    showRule.value = false
  } catch (e) {
    store.notify.fail(e, t('admin-rule-save-fail'))
  }
}

async function saveCategory(body: Record<string, unknown>) {
  try {
    await store.save('/api/admin/rules/categories', body, body.id as string | undefined)
    showCategory.value = false
  } catch (e) {
    store.notify.fail(e, t('admin-cat-save-fail'))
  }
}
</script>

<template>
  <NoroShell :title="t('admin-rules-title')" :subtitle="t('admin-rules-subtitle')">
    <template #actions>
      <AtomButton variant="dark" icon="i-lucide-refresh-cw" :loading="pending" @click="store.load()">
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
      <AtomButton v-if="canEdit" variant="secondary" icon="i-lucide-folder-plus" @click="actions.newSection(null)">
        {{ t('admin-rules-btn-section') }}
      </AtomButton>
      <AtomButton v-if="canEdit" variant="primary" icon="i-lucide-plus" @click="actions.newRule(null)">
        {{ t('admin-rules-btn-rule') }}
      </AtomButton>
    </template>

    <div class="grid gap-4">
      <div class="noro-panel flex flex-col gap-3 p-4 lg:flex-row lg:items-center">
        <label class="relative flex-1">
          <UIcon
            name="i-lucide-search"
            class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[var(--noro-muted)]"
          />
          <input v-model="search" class="noro-input w-full !pl-10" :placeholder="t('admin-rules-search-placeholder')">
        </label>
        <label class="flex items-center gap-2">
          <span class="noro-label">{{ t('admin-rules-scope') }}</span>
          <NoroSelect v-model="scope" class="w-56">
            <option value="">{{ t('admin-rules-scope-all') }}</option>
            <option v-for="s in servers" :key="s.id" :value="s.id">{{ t('admin-rule-server-only', { name: s.name }) }}</option>
          </NoroSelect>
        </label>
        <NuxtLink to="/rules" target="_blank" class="shrink-0">
          <AtomButton variant="ghost" icon="i-lucide-external-link">{{ t('admin-rules-public-page') }}</AtomButton>
        </NuxtLink>
      </div>

      <EmptyState
        v-if="!pending && !tree.length && !loose.length"
        icon="i-lucide-book-open"
        :title="t('admin-rules-empty-title')"
        :text="t('admin-rules-empty-text')"
      />

      <AdminRulesTree v-for="node in tree" :key="node.category.id" :node="node" />

      <section v-if="loose.length" class="noro-panel p-4">
        <header class="flex items-center gap-3">
          <h2 class="text-lg font-black text-[var(--noro-text)]">{{ t('admin-rules-other-section') }}</h2>
          <span class="noro-label">{{ t('admin-rules-outside-section') }}</span>
        </header>
        <div class="mt-4 grid gap-2">
          <AdminRuleRow v-for="rule in loose" :key="rule.id" :rule="rule" />
        </div>
      </section>
    </div>

    <AdminRuleModal
      v-model="showRule"
      :draft="ruleDraft"
      :sanctions="ruleDraft.id ? store.sanctionsOf(ruleDraft.id) : []"
      :categories="categories"
      :servers="servers"
      @save="saveRule"
    />
    <AdminRuleCategoryModal
      v-model="showCategory"
      :draft="categoryDraft"
      :categories="categories"
      :servers="servers"
      @save="saveCategory"
    />
  </NoroShell>
</template>
