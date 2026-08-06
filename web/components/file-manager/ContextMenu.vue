<script setup lang="ts">
defineProps<{
  x: number
  y: number
  isFolder: boolean
  hasSelection: boolean
  isText?: boolean
  /** Текущий режим синхронизации цели — чтобы отметить его в подменю. */
  syncMode?: SyncMode
}>()

// Подменю раскрывается по наведению, как в проводнике.
const syncOpen = ref(false)

const emit = defineEmits<{
  open: []
  edit: []
  download: []
  rename: []
  copy: []
  delete: []
  newFolder: []
  upload: []
  'set-sync': [mode: SyncMode]
  close: []
}>()

function onClickOutside() {
  emit('close')
}

onMounted(() => {
  setTimeout(() => {
    document.addEventListener('click', onClickOutside, { once: true })
    document.addEventListener('contextmenu', onClickOutside, { once: true })
  }, 10)
})

onUnmounted(() => {
  document.removeEventListener('click', onClickOutside)
  document.removeEventListener('contextmenu', onClickOutside)
})
</script>

<template>
  <Teleport to="body">
    <div
      class="fm-ctx"
      :style="{ left: `${x}px`, top: `${y}px` }"
      @contextmenu.prevent
      @click.stop
    >
      <button v-if="isFolder" class="fm-ctx-item" @click="emit('open')">
        <UIcon name="i-lucide-folder-open" class="size-4" /> Open
      </button>
      <button v-if="!isFolder && isText" class="fm-ctx-item" @click="emit('edit')">
        <UIcon name="i-lucide-file-edit" class="size-4" /> Edit
      </button>
      <button v-if="!isFolder" class="fm-ctx-item" @click="emit('download')">
        <UIcon name="i-lucide-download" class="size-4" /> Download
      </button>
      <button v-if="hasSelection" class="fm-ctx-item" @click="emit('rename')">
        <UIcon name="i-lucide-pencil" class="size-4" /> Rename
      </button>
      <button v-if="hasSelection" class="fm-ctx-item" @click="emit('copy')">
        <UIcon name="i-lucide-copy" class="size-4" /> Copy Path
      </button>
      <template v-if="hasSelection">
        <div class="fm-ctx-sep" />
        <!-- Режим ставится и отсюда: на плитках метки нет, да и списком
             попадать в одну букву неудобно. -->
        <div class="relative" @mouseenter="syncOpen = true" @mouseleave="syncOpen = false">
          <button class="fm-ctx-item w-full justify-between">
            <span class="flex items-center gap-2">
              <UIcon name="i-lucide-refresh-cw" class="size-4" /> Sync mode
            </span>
            <UIcon name="i-lucide-chevron-right" class="size-3.5" />
          </button>
          <div v-if="syncOpen" class="fm-ctx-sub">
            <button
              v-for="mode in MODE_ORDER"
              :key="mode"
              class="fm-ctx-item"
              @click="emit('set-sync', mode)"
            >
              <UIcon
                :name="syncMode === mode ? 'i-lucide-check' : 'i-lucide-minus'"
                class="size-4"
                :class="syncMode === mode ? 'text-[var(--noro-cream)]' : 'opacity-30'"
              />
              {{ MODE_HINT[mode] }}
            </button>
          </div>
        </div>
      </template>

      <div class="fm-ctx-sep" />
      <button class="fm-ctx-item" @click="emit('newFolder')">
        <UIcon name="i-lucide-folder-plus" class="size-4" /> New Folder
      </button>
      <button class="fm-ctx-item" @click="emit('upload')">
        <UIcon name="i-lucide-upload" class="size-4" /> Upload Files
      </button>
      <template v-if="hasSelection">
        <div class="fm-ctx-sep" />
        <button class="fm-ctx-item danger" @click="emit('delete')">
          <UIcon name="i-lucide-trash-2" class="size-4" /> Delete
        </button>
      </template>
    </div>
  </Teleport>
</template>
