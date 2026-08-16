<script setup lang="ts">
/**
 * Выпадающий список.
 *
 * Отдельным компонентом, а не парой классов на месте: нативный `select` не
 * наследует line-height от `.noro-input`, и высота расходилась с соседними
 * полями и кнопками в каждом месте по-своему.
 */
defineProps<{ disabled?: boolean }>()
const model = defineModel<string | number>({ required: true })
</script>

<template>
  <select v-model="model" class="noro-input noro-select" :disabled="disabled">
    <slot />
  </select>
</template>
