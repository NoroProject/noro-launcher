<script setup lang="ts">
/**
 * Выбор правила для наказания: поиск по коду и названию.
 *
 * Список, а не свободный ввод: код из свода должен попасть в наказание ровно
 * такой, какой в нём записан, — по нему потом разбирают жалобу.
 */
import type { Rule } from '~/types/rules'

const props = defineProps<{ rules: Rule[], serverId?: string }>()
const selected = defineModel<string>({ required: true })

const { t } = useT()
const search = ref('')

const found = computed(() => {
  const query = search.value.trim().toLowerCase()
  const list = query
    ? props.rules.filter(r => ruleMatches(r, query))
    : props.rules
  return list.slice(0, 8)
})

const picked = computed(() => props.rules.find(r => r.id === selected.value) || null)
</script>

<template>
  <div class="grid gap-2">
    <span class="noro-label">{{ t('admin-punish-rule-label') }}</span>

    <div v-if="picked" class="flex flex-wrap items-center gap-2 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-3 py-2">
      <span class="text-sm font-black text-[var(--noro-cream)]">{{ picked.code }}</span>
      <span class="min-w-0 flex-1 truncate text-sm font-bold text-[var(--noro-text)]">{{ picked.title }}</span>
      <AtomButton size="sm" variant="ghost" icon="i-lucide-x" :aria-label="t('admin-punish-rule-clear')"
                  @click="selected = ''" />
    </div>

    <template v-else>
      <label class="relative block">
        <UIcon
          name="i-lucide-search"
          class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[var(--noro-muted)]"
        />
        <input v-model="search" class="noro-input w-full !pl-10" :placeholder="t('admin-punish-rule-search')">
      </label>

      <div v-if="found.length" class="grid gap-1">
        <button
          v-for="rule in found"
          :key="rule.id"
          type="button"
          class="flex items-center gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-3 py-2 text-left transition hover:border-[var(--noro-cream)]"
          @click="selected = rule.id"
        >
          <span class="shrink-0 text-sm font-black text-[var(--noro-cream)]">{{ rule.code }}</span>
          <span class="min-w-0 flex-1 truncate text-sm font-bold text-[var(--noro-text)]">{{ rule.title }}</span>
        </button>
      </div>
      <p v-else class="text-sm font-medium text-[var(--noro-muted)]">
        {{ rules.length ? t('admin-punish-rule-nomatch') : t('admin-punish-rule-empty') }}
      </p>
    </template>
  </div>
</template>
