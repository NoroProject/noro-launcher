<script setup lang="ts">
defineProps<{
  icon?: string
  title: string
  text?: string
}>()
</script>

<template>
  <div class="noro-panel p-8 text-center">
    <div v-if="icon" class="mx-auto mb-4 grid size-14 place-items-center rounded-lg bg-[var(--noro-input)]">
      <UIcon :name="icon" class="size-7 text-[var(--noro-cream)]" />
    </div>
    <div class="text-lg font-black text-[var(--noro-text)]">{{ title }}</div>
    <p v-if="text" class="mx-auto mt-2 max-w-md text-sm font-medium text-[var(--noro-muted)]">{{ text }}</p>
  </div>
</template>
