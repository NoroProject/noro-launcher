<script setup lang="ts">
import type { PermissionSuggestion } from '~/types/permissions'

const props = defineProps<{
  modelValue: string
  suggestions: PermissionSuggestion[]
  loading?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  submit: []
}>()

const open = ref(false)
const active = ref(0)

const filtered = computed(() => {
  const needle = props.modelValue.trim().toLowerCase()
  if (!needle) return props.suggestions.slice(0, 40)
  return props.suggestions
    .filter(item => item.node.toLowerCase().includes(needle)
      || (item.label || '').toLowerCase().includes(needle))
    .slice(0, 40)
})

watch(filtered, () => {
  active.value = 0
})

function onInput(event: Event) {
  emit('update:modelValue', (event.target as HTMLInputElement).value)
  open.value = true
}

function pick(node: string) {
  emit('update:modelValue', node)
  open.value = false
}

/** Каталог неполный по своей природе: Enter без подсветки отправляет ручной ввод. */
function onEnter() {
  const hit = open.value ? filtered.value[active.value] : undefined
  if (hit) {
    pick(hit.node)
    return
  }
  open.value = false
  emit('submit')
}

function step(delta: number) {
  open.value = true
  const total = filtered.value.length
  if (!total) return
  active.value = (active.value + delta + total) % total
}
</script>

<template>
  <div class="relative">
    <input
      :value="modelValue"
      class="noro-input !pr-10 font-mono"
      placeholder="noro.admin.users"
      spellcheck="false"
      autocomplete="off"
      @input="onInput"
      @focus="open = true"
      @blur="open = false"
      @keydown.down.prevent="step(1)"
      @keydown.up.prevent="step(-1)"
      @keydown.enter.prevent="onEnter"
      @keydown.esc="open = false"
    >
    <UIcon
      :name="loading ? 'i-lucide-loader-circle' : 'i-lucide-search'"
      class="pointer-events-none absolute right-3 top-1/2 size-4 -translate-y-1/2 text-[var(--noro-muted)]"
      :class="loading ? 'animate-spin' : ''"
    />

    <div
      v-if="open"
      class="absolute z-50 mt-2 max-h-64 w-full overflow-auto rounded-lg border border-[var(--noro-border)] bg-[var(--noro-panel-2)] p-1"
      @mousedown.prevent
    >
      <button
        v-for="(item, index) in filtered"
        :key="item.node"
        type="button"
        class="flex w-full items-center gap-3 rounded px-3 py-2 text-left transition-colors duration-100"
        :class="index === active ? 'bg-[var(--noro-input)]' : 'hover:bg-[var(--noro-input)]'"
        @click="pick(item.node)"
      >
        <span class="min-w-0 flex-1">
          <span class="block truncate font-mono text-xs text-[var(--noro-cream)]">{{ item.node }}</span>
          <span v-if="item.label" class="block truncate text-xs text-[var(--noro-muted)]">{{ item.label }}</span>
        </span>
        <span
          class="shrink-0 rounded px-2 py-1 text-xs font-bold uppercase"
          :class="item.source === 'game'
            ? 'bg-[var(--noro-green)]/10 text-[var(--noro-green)]'
            : 'bg-[var(--noro-blue)]/10 text-[var(--noro-blue)]'"
        >{{ item.source }}</span>
      </button>

      <p v-if="!filtered.length" class="px-3 py-2 text-xs text-[var(--noro-muted)]">
        Not in the catalog. Press Enter to grant it anyway.
      </p>
    </div>
  </div>
</template>
