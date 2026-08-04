<script setup lang="ts">
interface Crumb { name: string; path: string }

defineProps<{
  breadcrumb: Crumb[]
  search: string
  viewMode: 'list' | 'grid'
  loading: boolean
}>()

const emit = defineEmits<{
  'update:search': [value: string]
  navigate: [path: string]
  back: []
  'set-view': [mode: 'list' | 'grid']
  'new-folder': []
  upload: []
  refresh: []
}>()
</script>

<template>
  <div class="fm-toolbar">
    <!-- Back button -->
    <button
      class="fm-icon-btn"
      title="Go up"
      :disabled="breadcrumb.length <= 1"
      @click="emit('back')"
    >
      <UIcon name="i-lucide-arrow-left" class="size-4" />
    </button>

    <!-- Breadcrumb -->
    <nav class="fm-breadcrumb flex-1 min-w-0">
      <template v-for="(b, i) in breadcrumb" :key="b.path">
        <span v-if="i > 0" class="fm-sep">/</span>
        <button @click="emit('navigate', b.path)">{{ b.name }}</button>
      </template>
    </nav>

    <!-- Search -->
    <input
      class="fm-search"
      placeholder="Search…"
      :value="search"
      @input="emit('update:search', ($event.target as HTMLInputElement).value)"
    />

    <!-- View toggle -->
    <button
      class="fm-icon-btn"
      :class="{ active: viewMode === 'list' }"
      title="List view"
      @click="emit('set-view', 'list')"
    >
      <UIcon name="i-lucide-list" class="size-4" />
    </button>
    <button
      class="fm-icon-btn"
      :class="{ active: viewMode === 'grid' }"
      title="Grid view"
      @click="emit('set-view', 'grid')"
    >
      <UIcon name="i-lucide-layout-grid" class="size-4" />
    </button>

    <div class="w-px h-5 bg-[var(--noro-border-soft)]" />

    <!-- Actions -->
    <button class="fm-icon-btn" title="New folder" @click="emit('new-folder')">
      <UIcon name="i-lucide-folder-plus" class="size-4" />
    </button>
    <label class="fm-icon-btn" title="Upload files">
      <UIcon name="i-lucide-upload" class="size-4" />
      <input type="file" multiple class="hidden" @change="emit('upload')" />
    </label>
    <button
      class="fm-icon-btn"
      title="Refresh"
      :class="{ 'animate-spin': loading }"
      @click="emit('refresh')"
    >
      <UIcon name="i-lucide-refresh-cw" class="size-4" />
    </button>
  </div>
</template>
