<script setup lang="ts">
/**
 * Оглавление свода. Строится из того же дерева, что и текст, поэтому при
 * поиске в нём остаются только разделы, где что-то нашлось.
 */
import type { RuleNode } from '~/types/rules'

defineProps<{ nodes: RuleNode[], loose: number }>()

const { t } = useT()

/** Плоский список с уровнем: рекурсивный компонент ради отступа избыточен. */
function flatten(nodes: RuleNode[], level = 0): { node: RuleNode, level: number }[] {
  return nodes.flatMap(node => [{ node, level }, ...flatten(node.children, level + 1)])
}
</script>

<template>
  <nav class="noro-panel sticky top-6 max-h-[calc(100vh-48px)] overflow-y-auto noro-scroll p-4">
    <div class="noro-label mb-3">{{ t('web-rules-contents') }}</div>
    <ul class="grid gap-1">
      <li v-for="entry in flatten(nodes)" :key="entry.node.category.id">
        <!-- Три колонки, а не flex: иначе длинное название выдавливало
             счётчик правил за край панели. -->
        <a
          :href="`#section-${entry.node.category.id}`"
          class="grid grid-cols-[auto_1fr_auto] items-baseline gap-2 rounded-[var(--noro-r-sm)] px-2 py-1 text-sm font-bold text-[var(--noro-muted)] transition hover:bg-[var(--noro-input)] hover:text-[var(--noro-text)]"
          :style="{ paddingLeft: `${8 + entry.level * 12}px` }"
        >
          <span class="text-[var(--noro-blue)]">{{ entry.node.category.code }}</span>
          <span class="truncate">{{ entry.node.category.name }}</span>
          <span class="text-xs">{{ entry.node.total }}</span>
        </a>
      </li>
      <li v-if="loose">
        <a
          href="#section-loose"
          class="grid grid-cols-[1fr_auto] items-baseline gap-2 rounded-[var(--noro-r-sm)] px-2 py-1 text-sm font-bold text-[var(--noro-muted)] transition hover:bg-[var(--noro-input)] hover:text-[var(--noro-text)]"
        >
          <span class="truncate">{{ t('web-rules-other') }}</span>
          <span class="text-xs">{{ loose }}</span>
        </a>
      </li>
    </ul>
  </nav>
</template>
