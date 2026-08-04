<script setup lang="ts">
const props = withDefaults(defineProps<{
  modelValue: string
  label?: string
  placeholder?: string
  type?: string
  disabled?: boolean
  readonly?: boolean
  error?: string
  hint?: string
}>(), {
  type: 'text',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const value = computed({
  get: () => props.modelValue,
  set: (v: string) => emit('update:modelValue', v),
})
</script>

<template>
  <label class="block">
    <span v-if="label" class="noro-label">{{ label }}</span>
    <input
      v-model="value"
      :type="type"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      class="noro-input"
      :class="{ 'border-2 border-[var(--noro-danger)]': !!error }"
    />
    <p v-if="error" class="mt-1 text-xs text-[var(--noro-danger)]">{{ error }}</p>
    <p v-else-if="hint" class="mt-1 text-xs text-[var(--noro-muted)]">{{ hint }}</p>
  </label>
</template>
