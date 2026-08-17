<script setup lang="ts">
const props = defineProps<{
  x: number
  y: number
  isFolder: boolean
  hasSelection: boolean
  isText?: boolean
  syncMode?: SyncMode
  modeHint?: Record<SyncMode, string>
}>()

const syncOpen = ref(false)
const { t } = useT()

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
        <UIcon name="i-lucide-folder-open" class="size-4" /> {{ t('admin-fm-open') }}
      </button>
      <button v-if="!isFolder && isText" class="fm-ctx-item" @click="emit('edit')">
        <UIcon name="i-lucide-file-edit" class="size-4" /> {{ t('admin-fm-edit') }}
      </button>
      <button v-if="!isFolder" class="fm-ctx-item" @click="emit('download')">
        <UIcon name="i-lucide-download" class="size-4" /> {{ t('admin-fm-download') }}
      </button>
      <button v-if="hasSelection" class="fm-ctx-item" @click="emit('rename')">
        <UIcon name="i-lucide-pencil" class="size-4" /> {{ t('admin-fm-rename') }}
      </button>
      <button v-if="hasSelection" class="fm-ctx-item" @click="emit('copy')">
        <UIcon name="i-lucide-copy" class="size-4" /> {{ t('admin-fm-copy-path') }}
      </button>
      <template v-if="hasSelection">
        <div class="fm-ctx-sep" />
        <div class="relative" @mouseenter="syncOpen = true" @mouseleave="syncOpen = false">
          <button class="fm-ctx-item w-full justify-between">
            <span class="flex items-center gap-2">
              <UIcon name="i-lucide-refresh-cw" class="size-4" /> {{ t('admin-fm-sync-mode') }}
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
              {{ modeHint ? modeHint[mode] : mode }}
            </button>
          </div>
        </div>
      </template>

      <div class="fm-ctx-sep" />
      <button class="fm-ctx-item" @click="emit('newFolder')">
        <UIcon name="i-lucide-folder-plus" class="size-4" /> {{ t('admin-fm-new-folder') }}
      </button>
      <button class="fm-ctx-item" @click="emit('upload')">
        <UIcon name="i-lucide-upload" class="size-4" /> {{ t('admin-fm-upload') }}
      </button>
      <template v-if="hasSelection">
        <div class="fm-ctx-sep" />
        <button class="fm-ctx-item danger" @click="emit('delete')">
          <UIcon name="i-lucide-trash-2" class="size-4" /> {{ t('admin-fm-delete') }}
        </button>
      </template>
    </div>
  </Teleport>
</template>
