<script setup lang="ts">
/**
 * Шапка дела: кого разбирают и на каком счету он у сервера.
 *
 * Голова скина здесь не украшение — модератор ищет игрока в игре по внешности,
 * а не по строке ника, и опознать его на экране быстрее по скину.
 */
import type { CaseRow } from '~/types/cases'

const props = defineProps<{ item: CaseRow, punishments: number }>()
const { t } = useT()
const config = useRuntimeConfig()

const STATUS_TONE: Record<string, string> = {
  open: 'text-amber-400',
  in_review: 'text-[var(--noro-blue)]',
  resolved: 'text-emerald-400',
  rejected: 'text-[var(--noro-muted)]',
}

/** Рендер головы — тот же, что в поиске: один источник на всю админку. */
const head = computed(() => {
  const master = config.public.masterUrl || ''
  const name = props.item.target_name
  return name ? `${master}/api/textures/renders?mode=flat-head&scale=6&username=${encodeURIComponent(name)}` : null
})
</script>

<template>
  <section class="noro-panel grid min-w-0 content-start gap-4 p-4">
    <div class="flex flex-wrap items-center gap-4">
      <img
        v-if="head"
        :src="head"
        alt=""
        class="size-12 shrink-0 [image-rendering:pixelated]"
        @error="($event.target as HTMLImageElement).style.display = 'none'"
      >
      <div class="min-w-0 flex-1">
        <div class="flex items-baseline gap-3">
          <h2 class="noro-pixel truncate text-2xl uppercase text-[var(--noro-cream)]">
            {{ item.target_name || '—' }}
          </h2>
          <span class="shrink-0 font-mono text-xs text-[var(--noro-muted)]">{{ caseNumber(item.number) }}</span>
        </div>
        <p class="mt-1 text-xs text-[var(--noro-muted)]">
          {{ item.server_name || '—' }} · {{ t('admin-case-opened') }} {{ relativeDateT(item.opened_at, t) }}
          <template v-if="item.claimed_by_name">
            · {{ t('admin-cases-claimed-by') }} {{ item.claimed_by_name }}
          </template>
        </p>
      </div>
      <div class="text-right">
        <div class="noro-label">{{ t('admin-cases-status') }}</div>
        <div class="text-sm font-bold" :class="STATUS_TONE[item.status]">
          {{ t(`admin-cases-status-${item.status}`) }}
        </div>
      </div>
    </div>

    <!-- Три числа, по которым дело оценивают, не открывая: сколько жалоб, от
         скольких разных людей и чем уже кончилось. -->
    <dl class="grid grid-cols-3 self-start gap-px overflow-hidden border border-[var(--noro-border)] bg-[var(--noro-border)]">
      <div class="bg-[var(--noro-panel)] p-3">
        <dt class="noro-label">{{ t('admin-case-metric-reports') }}</dt>
        <dd class="noro-pixel text-xl text-white">{{ item.reports_count }}</dd>
      </div>
      <div class="bg-[var(--noro-panel)] p-3">
        <dt class="noro-label">{{ t('admin-case-metric-people') }}</dt>
        <dd class="noro-pixel text-xl text-white">{{ item.reporters_count }}</dd>
      </div>
      <div class="bg-[var(--noro-panel)] p-3">
        <dt class="noro-label">{{ t('admin-case-metric-punishments') }}</dt>
        <dd class="noro-pixel text-xl text-white">{{ punishments }}</dd>
      </div>
    </dl>

    <div v-if="item.verdict" class="border-t border-[var(--noro-border)] pt-3">
      <span class="noro-label">{{ t('admin-case-verdict') }}</span>
      <p class="text-sm text-[var(--noro-text)]">
        {{ t(`admin-case-verdict-${item.verdict}`) }}
        <span v-if="item.rule_code" class="text-[var(--noro-muted)]">· {{ t('admin-case-rule') }} {{ item.rule_code }}</span>
      </p>
    </div>
  </section>
</template>
