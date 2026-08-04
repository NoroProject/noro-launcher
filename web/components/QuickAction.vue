<script setup lang="ts">
defineProps<{
  to: string
  icon: string
  label: string
  primary?: boolean
}>()
</script>

<template>
  <NuxtLink
    :to="to"
    class="noro-btn flex gap-3"
    :class="primary ? 'noro-btn-primary' : 'noro-btn-secondary'"
  >
    <UIcon :name="icon" class="size-5" />
    <span>{{ label }}</span>
  </NuxtLink>
</template>
