<script setup lang="ts">
export interface NoroSelectOption {
  label: string
  value: string | number
  disabled?: boolean
}

defineProps<{
  disabled?: boolean
  options?: NoroSelectOption[]
}>()
const model = defineModel<string | number>({ required: true })
</script>

<template>
  <select v-model="model" class="noro-input noro-select" :disabled="disabled">
    <slot>
      <option v-for="opt in options" :key="opt.value" :value="opt.value" :disabled="opt.disabled">
        {{ opt.label }}
      </option>
    </slot>
  </select>
</template>
