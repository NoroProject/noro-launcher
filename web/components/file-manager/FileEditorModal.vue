<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
  buildId: string
  fileId: string
  fileName: string
  filePath: string
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const auth = useAuth()
const content = ref('')
const loading = ref(true)
const saving = ref(false)

async function loadContent() {
  loading.value = true
  try {
  const res = await auth.request(`/api/admin/builds/${props.buildId}/files/content?path=${encodeURIComponent(props.filePath)}`)
  content.value = res.content
  } catch (err) {
    console.error(err)
    alert('Failed to load file content')
  } finally {
    loading.value = false
  }
}

async function save() {
  saving.value = true
  try {
    await auth.request(`/api/admin/builds/${props.buildId}/files/content`, {
      method: 'PUT',
      body: {
        path: props.filePath,
        content: content.value
      }
    })
    emit('saved')
  } catch (err) {    console.error(err)
    alert('Failed to save file content')
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadContent()
  const onKey = (e: KeyboardEvent) => {
    if (e.key === 'Escape') emit('close')
    if ((e.metaKey || e.ctrlKey) && e.key === 's') {
      e.preventDefault()
      save()
    }
  }
  window.addEventListener('keydown', onKey)
  onUnmounted(() => window.removeEventListener('keydown', onKey))
})
</script>

<template>
  <div class="fixed inset-0 z-[100] bg-black/60 flex items-center justify-center backdrop-blur-sm p-4">
    <div class="noro-panel w-full max-w-5xl h-[80vh] flex flex-col overflow-hidden bg-[var(--noro-bg)] border-[var(--noro-border)]" @click.stop>
      <div class="flex items-center justify-between p-3 border-b border-[var(--noro-border-soft)]">
        <h3 class="font-bold text-[var(--noro-fg)] flex items-center gap-2">
          <UIcon name="i-lucide-file-edit" class="text-[var(--noro-blue)]" />
          Edit: {{ fileName }}
          <span class="text-xs font-normal text-[var(--noro-muted)]">{{ filePath }}</span>
        </h3>
        <div class="flex items-center gap-2">
          <AtomButton variant="primary"
            size="sm"
            :loading="saving"
            @click="save"
          >
            Save (Ctrl+S)
          </AtomButton>
          <AtomButton variant="ghost"
            size="sm"
            icon="i-lucide-x"
            @click="emit('close')"
          />
        </div>
      </div>
      <div class="flex-1 relative bg-[#1e1e1e]">
        <div v-if="loading" class="absolute inset-0 flex items-center justify-center bg-black/50 z-10">
          <UIcon name="i-lucide-loader-2" class="animate-spin size-8 text-[var(--noro-primary)]" />
        </div>
        <textarea
          v-model="content"
          class="w-full h-full p-4 bg-transparent text-[#d4d4d4] font-mono text-sm resize-none focus:outline-none"
          spellcheck="false"
          :disabled="loading || saving"
        ></textarea>
      </div>
    </div>
  </div>
</template>

<style scoped>
textarea {
  tab-size: 2;
  white-space: pre;
  overflow: auto;
}
textarea::-webkit-scrollbar {
  width: 14px;
  height: 14px;
}
textarea::-webkit-scrollbar-track {
  background: transparent;
}
textarea::-webkit-scrollbar-thumb {
  background-color: #424242;
  border: 4px solid #1e1e1e;
  border-radius: 8px;
}
</style>
