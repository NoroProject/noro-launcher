<script setup lang="ts">
/**
 * Вердикт разбора — три карточки, а не выпадающий список.
 *
 * Это решение о человеке, и оно должно стоить осознанного выбора: в списке
 * «подтвердилось» стоит первым и выбирается по инерции, а здесь три исхода
 * лежат рядом и подписаны последствием.
 */
const verdict = defineModel<'confirmed' | 'rejected' | 'insufficient'>({ required: true })
const { t } = useT()

const OPTIONS = [
  { value: 'confirmed', icon: 'i-lucide-check', tone: 'emerald' },
  { value: 'rejected', icon: 'i-lucide-x', tone: 'muted' },
  { value: 'insufficient', icon: 'i-lucide-help-circle', tone: 'amber' },
] as const

const TONE: Record<string, string> = {
  emerald: 'border-emerald-400 bg-emerald-400/10 text-emerald-400',
  muted: 'border-[var(--noro-muted)] bg-[var(--noro-muted)]/10 text-[var(--noro-text)]',
  amber: 'border-amber-400 bg-amber-400/10 text-amber-400',
}
</script>

<template>
  <div class="grid gap-2">
    <button
      v-for="option in OPTIONS"
      :key="option.value"
      type="button"
      class="flex items-start gap-3 border p-3 text-left transition"
      :class="verdict === option.value
        ? TONE[option.tone]
        : 'border-[var(--noro-border)] text-[var(--noro-muted)] hover:border-[var(--noro-blue)]'"
      @click="verdict = option.value"
    >
      <UIcon :name="option.icon" class="mt-0.5 size-4 shrink-0" />
      <span class="grid gap-0.5">
        <span class="text-sm font-bold uppercase">{{ t(`admin-case-verdict-${option.value}`) }}</span>
        <span class="text-xs leading-5 text-[var(--noro-muted)]">
          {{ t(`admin-case-verdict-${option.value}-hint`) }}
        </span>
      </span>
    </button>
  </div>
</template>
