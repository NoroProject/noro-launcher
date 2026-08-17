<script setup lang="ts">
/** Переключатель языка. Выбор запоминается в cookie и переживает перезаход. */
const { locale, setLocale } = useT()
</script>

<template>
  <div class="flex h-9 items-center gap-1 rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] p-1">
    <button
      v-for="loc in LOCALES"
      :key="loc.code"
      type="button"
      class="flex h-7 items-center justify-center rounded-[var(--noro-r-sm)] px-2.5 text-xs font-bold uppercase tracking-wider transition"
      :class="locale === loc.code
        ? 'bg-[var(--noro-panel-2)] text-[var(--noro-cream)]'
        : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
      :aria-pressed="locale === loc.code"
      :title="loc.label"
      @click="setLocale(loc.code)"
    >{{ loc.short }}</button>
  </div>
</template>
