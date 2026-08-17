<script setup lang="ts">
/**
 * Раздел свода в админке: правила внутри и вложенные разделы под ними.
 * Компонент рекурсивный — глубина дерева ничем не ограничена.
 */
import type { RuleNode } from '~/types/rules'
import { RULE_ACTIONS } from '~/types/rules'

withDefaults(defineProps<{ node: RuleNode, level?: number }>(), { level: 0 })

const actions = inject(RULE_ACTIONS)!
</script>

<template>
  <section class="noro-panel p-4">
    <header class="flex flex-wrap items-center gap-3">
      <span v-if="node.category.code" class="noro-chip px-2 py-1 text-xs font-bold text-[var(--noro-blue)]">
        {{ node.category.code }}
      </span>
      <h2 class="font-black text-[var(--noro-text)]" :class="level ? 'text-base' : 'text-lg'">
        {{ node.category.name }}
      </h2>
      <span v-if="node.category.server_id" class="noro-chip px-2 py-1 text-xs font-bold text-[var(--noro-cream)]">
        {{ actions.serverName(node.category.server_id) }}
      </span>
      <span class="noro-label">{{ node.total }} {{ node.total === 1 ? 'rule' : 'rules' }}</span>

      <div v-if="actions.canEdit || actions.canDelete" class="ml-auto flex items-center gap-1">
        <template v-if="actions.canEdit">
          <AtomButton size="sm" variant="ghost" icon="i-lucide-chevron-up" aria-label="Move section up"
                      @click="actions.move('categories', node.category.id, -1)" />
          <AtomButton size="sm" variant="ghost" icon="i-lucide-chevron-down" aria-label="Move section down"
                      @click="actions.move('categories', node.category.id, 1)" />
          <AtomButton size="sm" variant="secondary" icon="i-lucide-plus" @click="actions.newRule(node.category)">
            Rule
          </AtomButton>
          <AtomButton size="sm" variant="ghost" icon="i-lucide-folder-plus" aria-label="Add subsection"
                      @click="actions.newSection(node.category)" />
          <AtomButton size="sm" variant="ghost" icon="i-lucide-pencil" aria-label="Edit section"
                      @click="actions.editCategory(node.category)" />
        </template>
        <AtomButton v-if="actions.canDelete" size="sm" variant="ghost" icon="i-lucide-trash-2"
                    aria-label="Delete section" @click="actions.removeCategory(node.category)" />
      </div>
    </header>

    <p v-if="node.category.description" class="mt-2 text-sm font-medium text-[var(--noro-muted)]">
      {{ node.category.description }}
    </p>

    <div v-if="node.rules.length" class="mt-4 grid gap-2">
      <AdminRuleRow v-for="rule in node.rules" :key="rule.id" :rule="rule" />
    </div>
    <p v-else class="mt-4 text-sm font-medium text-[var(--noro-muted)]">
      No rules here yet.
    </p>

    <div v-if="node.children.length" class="mt-4 grid gap-3 border-l-2 border-[var(--noro-border)] pl-4">
      <AdminRulesTree
        v-for="child in node.children"
        :key="child.category.id"
        :node="child"
        :level="level + 1"
      />
    </div>
  </section>
</template>
