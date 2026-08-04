<script setup lang="ts">
import type { BuildFileRow } from '~/types/api'

const props = defineProps<{
  files: BuildFileRow[] | null | undefined
  buildId: string
}>()

const emit = defineEmits<{
  'open-manager': []
}>()

const preview = computed(() => {
  if (!props.files?.length) return []
  // Show first 10 non-core files for useful preview (fills space better)
  return props.files
    .filter(f => !f.path.startsWith('assets/') && !f.path.startsWith('libraries/'))
    .slice(0, 10)
})

const total = computed(() => props.files?.length || 0)
const hasCore = computed(() => total.value > preview.value.length)
</script>

<template>
  <div class="noro-panel p-4">
    <div class="flex items-center justify-between mb-3">
      <div>
        <div class="font-bold text-[var(--noro-text)]">Build Files</div>
        <div class="text-xs text-[var(--noro-muted)]">
          {{ total }} files{{ hasCore ? ' (core assets hidden in preview)' : '' }}
        </div>
      </div>
      <AtomButton variant="primary" @click="emit('open-manager')">
        Open File Manager
      </AtomButton>
    </div>

    <div v-if="preview.length" class="space-y-1 text-xs font-mono border border-[var(--noro-border)] rounded p-2 bg-black/10 max-h-[220px] overflow-auto">
      <div
        v-for="f in preview"
        :key="f.id"
        class="flex items-center justify-between text-[var(--noro-muted)] hover:text-[var(--noro-text)]"
      >
        <span class="truncate">{{ f.path }}</span>
        <span class="ml-2 shrink-0 text-[10px] opacity-70">{{ (f.size / 1024).toFixed(0) }} KB</span>
      </div>
    </div>

    <div v-else class="text-xs text-[var(--noro-muted)] italic p-2">
      No files yet. Use the panels on the right to add.
    </div>
  </div>
</template>
