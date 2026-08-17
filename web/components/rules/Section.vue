<script setup lang="ts">
/**
 * Раздел свода вместе с вложенными. Компонент рекурсивный: глубина дерева в
 * разметке ничем не ограничена — раньше третий уровень просто не выводился,
 * и его правила не существовали для читателя.
 */
import type { RuleNode, RuleSanction } from '~/types/rules'

const props = withDefaults(
  defineProps<{ node: RuleNode, level?: number, sanctions: RuleSanction[] }>(),
  { level: 0 },
)

const { t } = useT()

const byRule = computed(() => {
  const map = new Map<string, RuleSanction[]>()
  for (const sanction of props.sanctions) {
    map.set(sanction.rule_id, [...(map.get(sanction.rule_id) || []), sanction])
  }
  return map
})
</script>

<template>
  <section :id="`section-${node.category.id}`" class="scroll-mt-24">
    <header
      class="flex flex-wrap items-baseline gap-x-3 gap-y-1 border-b border-[var(--noro-border)] pb-3"
      :class="level ? '' : 'pt-2'"
    >
      <span
        v-if="node.category.code"
        class="noro-pixel shrink-0 text-[var(--noro-blue)]"
        :class="level ? 'text-sm' : 'text-base'"
      >{{ node.category.code }}</span>
      <h2
        class="font-black text-[var(--noro-cream)]"
        :class="level ? 'text-lg' : 'text-2xl'"
      >{{ node.category.name }}</h2>
      <span class="noro-label ml-auto">{{ t('web-rules-count', { count: node.total }) }}</span>
    </header>

    <p
      v-if="node.category.description"
      class="mt-3 text-sm font-medium leading-6 text-[var(--noro-muted)]"
    >{{ node.category.description }}</p>

    <div v-if="node.rules.length" class="mt-4 grid gap-3">
      <RulesCard v-for="rule in node.rules" :key="rule.id" :rule="rule" :sanctions="byRule.get(rule.id) || []" />
    </div>

    <!-- Вложенные разделы с отбивкой слева: вложенность должна читаться
         глазом, а не только по номеру пункта. -->
    <div v-if="node.children.length" class="mt-6 grid gap-6 border-l-2 border-[var(--noro-border)] pl-4">
      <RulesSection
        v-for="child in node.children"
        :key="child.category.id"
        :node="child"
        :level="level + 1"
        :sanctions="sanctions"
      />
    </div>
  </section>
</template>
