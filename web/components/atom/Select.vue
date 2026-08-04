<script setup lang="ts">
const props = withDefaults(defineProps<{
  modelValue: string
  label: string
  options: string[]
  placeholder?: string
  loading?: boolean
  disabled?: boolean
}>(), {
  placeholder: 'Search or type',
  loading: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const open = ref(false)
const query = ref('')
const dropUp = ref(false)

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  const selected = props.modelValue.trim().toLowerCase()
  if (!q || q === selected) return props.options.slice(0, 24)
  return props.options.filter(option => option.toLowerCase().includes(q)).slice(0, 24)
})

watch(() => props.modelValue, value => {
  query.value = value || ''
}, { immediate: true })

function select(value: string) {
  emit('update:modelValue', value)
  query.value = value
  open.value = false
}

function commit() {
  const value = query.value.trim()
  emit('update:modelValue', value)
  open.value = false
}

function focusInput(event: FocusEvent) {
  open.value = true
  const input = event.target as HTMLInputElement
  updateDropDirection(input)
  input.select()
}

function updateDropDirection(input: HTMLInputElement) {
  const rect = input.getBoundingClientRect()
  const below = window.innerHeight - rect.bottom
  dropUp.value = below < 280 && rect.top > below
}
</script>

<template>
  <label class="relative block">
    <span class="noro-label">{{ label }}</span>
    <div class="relative">
      <input
        v-model="query"
        class="noro-input pr-10"
        :placeholder="placeholder"
        :disabled="disabled"
        @focus="focusInput"
        @input="open = true"
        @blur="commit"
      >
      <UIcon
        :name="loading ? 'i-lucide-loader-circle' : 'i-lucide-chevron-down'"
        class="pointer-events-none absolute right-3 top-1/2 size-4 -translate-y-1/2 text-[var(--noro-muted)]"
        :class="loading ? 'animate-spin' : ''"
      />
    </div>
    <div
      v-if="open"
      class="absolute z-50 max-h-64 w-full overflow-auto rounded-lg bg-[var(--noro-input)] p-1"
      :class="dropUp ? 'bottom-full mb-2' : 'mt-2'"
      @mousedown.prevent
    >
      <button
        v-for="option in filtered"
        :key="option"
        type="button"
        class="block w-full rounded px-3 py-1.5 text-left text-sm text-[var(--noro-text)] hover:bg-[var(--noro-cream)] hover:text-[var(--noro-on-cream)] focus:bg-[var(--noro-cream)] focus:text-[var(--noro-on-cream)]"
        @click="select(option)"
      >
        {{ option }}
      </button>
      <div v-if="!filtered.length" class="px-3 py-3 text-sm text-[var(--noro-muted)]">
        No matches. Press tab to keep typed value.
      </div>
    </div>
  </label>
</template>
