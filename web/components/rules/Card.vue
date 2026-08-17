<script setup lang="ts">
/**
 * Одно правило. Код — якорь: ссылка вида `/rules#1.1` открывает страницу
 * ровно на том пункте, на который сослались в бане.
 */
import type { Rule, RuleSanction } from '~/types/rules'

const props = defineProps<{ rule: Rule, sanctions?: RuleSanction[] }>()

const notify = useNotify()
const { t } = useT()
const anchor = computed(() => `rule-${props.rule.code}`)

function copyLink() {
  const url = `${window.location.origin}${window.location.pathname}#${anchor.value}`
  navigator.clipboard.writeText(url)
  notify.info(t('web-rules-link-copied'), t('web-rules-link-copied-body', { code: props.rule.code }))
}
</script>

<template>
  <article
    :id="anchor"
    class="noro-card group scroll-mt-24 hover:border-[color-mix(in_srgb,var(--noro-cream)_36%,var(--noro-border))]"
  >
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div class="flex min-w-0 items-start gap-3">
        <span
          class="noro-chip shrink-0 px-2 py-1 text-xs font-bold text-[var(--noro-cream)]"
        >{{ rule.code }}</span>
        <h3 class="min-w-0 pt-1 text-base font-black text-[var(--noro-text)]">{{ rule.title }}</h3>
      </div>

      <div class="flex items-center gap-2">
        <!-- Вариантов бывает несколько: «первое нарушение» и «повторное»
             живут в одном правиле и должны быть видны игроку оба. -->
        <span
          v-for="sanction in sanctions"
          :key="sanction.id"
          class="noro-chip whitespace-nowrap px-2 py-1 text-xs font-bold text-[var(--noro-amber)]"
          :title="sanction.label || t('web-sanction-possible')"
        >{{ formatSanction(sanction, t) }}</span>
        <button
          type="button"
          class="rounded-[var(--noro-r-sm)] p-1 text-[var(--noro-muted)] opacity-0 transition hover:bg-[var(--noro-input)] hover:text-[var(--noro-text)] focus-visible:opacity-100 group-hover:opacity-100"
          :aria-label="t('web-rules-copy-link')"
          @click="copyLink"
        >
          <UIcon name="i-lucide-link" class="size-4" />
        </button>
      </div>
    </div>

    <p
      v-if="rule.description"
      class="mt-3 whitespace-pre-line text-sm font-medium leading-6 text-[var(--noro-muted)]"
    >{{ rule.description }}</p>
  </article>
</template>
