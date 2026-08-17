<script setup lang="ts">
/** Строка правила в админском дереве. */
import type { Rule } from '~/types/rules'
import { RULE_ACTIONS } from '~/types/rules'

const props = defineProps<{ rule: Rule }>()

const actions = inject(RULE_ACTIONS)!
const { t } = useT()
const sanctions = computed(() => actions.sanctionsOf(props.rule.id))
</script>

<template>
  <div class="flex flex-wrap items-center gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-3 py-2">
    <span class="shrink-0 text-sm font-black text-[var(--noro-cream)]">{{ rule.code }}</span>
    <span class="min-w-0 flex-1 truncate text-sm font-bold text-[var(--noro-text)]">{{ rule.title }}</span>

    <span
      v-for="sanction in sanctions"
      :key="sanction.id"
      class="noro-chip shrink-0 px-2 py-1 text-xs font-bold text-[var(--noro-amber)]"
      :title="sanction.label"
    >{{ formatSanction(sanction, t) }}</span>
    <span
      v-if="rule.server_id"
      class="noro-chip shrink-0 px-2 py-1 text-xs font-bold text-[var(--noro-cream)]"
    >{{ actions.serverName(rule.server_id) }}</span>

    <div v-if="actions.canEdit || actions.canDelete" class="flex shrink-0 items-center gap-1">
      <template v-if="actions.canEdit">
        <AtomButton size="sm" variant="ghost" icon="i-lucide-chevron-up" aria-label="Move rule up"
                    @click="actions.move('rules', rule.id, -1)" />
        <AtomButton size="sm" variant="ghost" icon="i-lucide-chevron-down" aria-label="Move rule down"
                    @click="actions.move('rules', rule.id, 1)" />
        <AtomButton size="sm" variant="ghost" icon="i-lucide-pencil" aria-label="Edit rule"
                    @click="actions.editRule(rule)" />
      </template>
      <AtomButton v-if="actions.canDelete" size="sm" variant="ghost" icon="i-lucide-trash-2"
                  aria-label="Delete rule" @click="actions.removeRule(rule)" />
    </div>
  </div>
</template>
