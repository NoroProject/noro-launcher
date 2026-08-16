<script setup lang="ts">
/**
 * Пояснение к разделу.
 *
 * Не `UAlert color="neutral"`: тот приходит со светлой темой Nuxt UI и выпадал
 * серым пятном на тёмном интерфейсе.
 */
withDefaults(defineProps<{ icon?: string }>(), { icon: 'i-lucide-info' })
</script>

<template>
  <div class="flex gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3">
    <UIcon :name="icon" class="mt-0.5 size-4 shrink-0 text-[var(--noro-blue)]" />
    <p class="text-xs leading-relaxed text-[var(--noro-muted)]">
      <slot />
    </p>
  </div>
</template>
