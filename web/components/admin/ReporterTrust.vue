<script setup lang="ts">
/**
 * Вес слова жалобщика: сколько его жалоб подтвердилось.
 *
 * Полоской, а не строкой «1 из 4»: в списке из трёх жалоб цифры сливаются, а
 * доля видна боковым зрением. Нерассмотренные показаны отдельным серым куском
 * — это не «не подтвердилось», а «ещё неизвестно», и путать их нельзя.
 */
import type { ReporterStats } from '~/types/cases'

const props = defineProps<{ stats?: ReporterStats }>()
const { t } = useT()

const total = computed(() => props.stats?.total ?? 0)
const parts = computed(() => {
  const s = props.stats
  if (!s?.total) return null
  const pending = Math.max(0, s.total - s.confirmed - s.rejected)
  return {
    confirmed: (s.confirmed / s.total) * 100,
    rejected: (s.rejected / s.total) * 100,
    pending: (pending / s.total) * 100,
  }
})
</script>

<template>
  <span v-if="parts" class="inline-flex items-center gap-2" :title="t('admin-case-trust-title')">
    <span class="flex h-1.5 w-16 overflow-hidden bg-[var(--noro-border)]">
      <span class="bg-emerald-400" :style="{ width: `${parts.confirmed}%` }" />
      <span class="bg-[var(--noro-danger)]/60" :style="{ width: `${parts.rejected}%` }" />
      <span class="bg-[var(--noro-muted)]/40" :style="{ width: `${parts.pending}%` }" />
    </span>
    <span class="text-xs text-[var(--noro-muted)]">
      {{ t('admin-case-reporter-stats', { confirmed: stats!.confirmed, total }) }}
    </span>
  </span>
</template>
