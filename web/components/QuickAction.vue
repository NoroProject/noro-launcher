<script setup lang="ts">
defineProps<{
  to: string
  icon: string
  label: string
  primary?: boolean
}>()
</script>

<template>
  <AtomButton
    :variant="primary ? 'primary' : 'secondary'"
    :icon="icon"
    :to="to"
    class="flex gap-3"
  >
    <span>{{ label }}</span>
  </AtomButton>
</template>
