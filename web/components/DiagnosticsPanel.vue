<script setup lang="ts">
import type { DiagnosticCheck } from '~/types/settings'

defineProps<{ checks: DiagnosticCheck[] }>()
const { t } = useT()

const CLASS: Record<DiagnosticCheck['level'], string> = {
  ok: 'text-[var(--noro-blue)]',
  warn: 'text-[var(--noro-cream)]',
  fail: 'text-[var(--noro-magenta)]',
}

const ICON: Record<DiagnosticCheck['level'], string> = {
  ok: 'i-lucide-circle-check',
  warn: 'i-lucide-triangle-alert',
  fail: 'i-lucide-circle-x',
}
</script>

<template>
  <section class="noro-panel h-fit p-6">
    <h2 class="mb-1 text-lg font-black text-[var(--noro-text)]">{{ t('admin-diag-title') }}</h2>
    <p class="mb-4 text-xs text-[var(--noro-muted)]">
      {{ t('admin-diag-panel-desc') }}
    </p>
    <ul class="grid gap-3">
      <li v-for="c in checks" :key="c.id" class="flex gap-3">
        <UIcon :name="ICON[c.level]" class="mt-0.5 size-4 shrink-0" :class="CLASS[c.level]" />
        <div>
          <div class="text-xs font-bold text-[var(--noro-text)]">{{ c.title }}</div>
          <div class="text-xs text-[var(--noro-muted)]">{{ c.detail }}</div>
        </div>
      </li>
    </ul>
  </section>
</template>
