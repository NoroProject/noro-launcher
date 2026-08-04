<script setup lang="ts">
import type { FmItem } from './List.vue'

defineProps<{
  items: FmItem[]
  selected: Set<string>
  renamingId: string | null
  renameValue: string
}>()

const emit = defineEmits<{
  select: [id: string, ev: MouseEvent]
  open: [item: FmItem]
  context: [item: FmItem, ev: MouseEvent]
  'bg-context': [ev: MouseEvent]
  'update:renameValue': [v: string]
  'rename-commit': []
  'rename-cancel': []
}>()

function itemKey(it: FmItem) {
  return it.type === 'folder' ? `d:${it.name}` : `f:${it.id}`
}

function fmtSize(b?: number) {
  if (b == null) return ''
  if (b < 1024) return `${b} B`
  if (b < 1048576) return `${(b / 1024).toFixed(1)} KB`
  return `${(b / 1048576).toFixed(1)} MB`
}

function iconForKind(it: FmItem) {
  if (it.type === 'folder') return 'i-lucide-folder'
  const ext = it.name.split('.').pop()?.toLowerCase() || ''
  const map: Record<string, string> = {
    jar: 'i-lucide-package',
    json: 'i-lucide-braces',
    toml: 'i-lucide-settings',
    cfg: 'i-lucide-settings',
    png: 'i-lucide-image',
    jpg: 'i-lucide-image',
    txt: 'i-lucide-file-text',
    log: 'i-lucide-file-text',
  }
  return map[ext] || 'i-lucide-file'
}

function iconColor(it: FmItem) {
  if (it.type === 'folder') return 'text-[var(--noro-amber)]'
  return 'text-[var(--noro-blue)]'
}
</script>

<template>
  <div
    class="fm-grid"
    @contextmenu.prevent="emit('bg-context', $event)"
  >
    <div
      v-for="it in items"
      :key="itemKey(it)"
      class="fm-grid-card"
      :class="{ 'fm-selected': selected.has(itemKey(it)) }"
      @click="emit('select', itemKey(it), $event)"
      @dblclick="emit('open', it)"
      @contextmenu.prevent.stop="emit('context', it, $event)"
    >
      <UIcon
        :name="iconForKind(it)"
        :class="['size-10', iconColor(it)]"
      />
      <template v-if="renamingId === itemKey(it)">
        <input
          class="fm-rename-input text-center"
          :value="renameValue"
          @input="emit('update:renameValue', ($event.target as HTMLInputElement).value)"
          @keydown.enter="emit('rename-commit')"
          @keydown.escape="emit('rename-cancel')"
          @blur="emit('rename-commit')"
          autofocus
        />
      </template>
      <template v-else>
        <span class="fm-grid-name">{{ it.name }}</span>
        <span class="text-[0.65rem] text-[var(--noro-muted)]">
          {{ it.type === 'folder' ? `${it.count} items` : fmtSize(it.size) }}
        </span>
      </template>
    </div>
    <div
      v-if="!items.length"
      class="col-span-full text-center text-[var(--noro-muted)] py-12"
    >
      Empty folder
    </div>
  </div>
</template>
