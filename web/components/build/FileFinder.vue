<script setup lang="ts">
import type { BuildFileRow } from '~/types/api'

const props = defineProps<{ buildId: string }>()
const emit = defineEmits<{ refresh: [] }>()

const auth = useAuth()
const files = ref<BuildFileRow[]>([])
const pending = ref(false)
const search = ref('')
const busy = ref<string | null>(null)

async function load() {
  pending.value = true
  try {
    files.value = await auth.request<BuildFileRow[]>(`/api/admin/builds/${props.buildId}/files`)
  } finally {
    pending.value = false
  }
}

onMounted(load)

const filtered = computed(() => {
  const q = search.value.toLowerCase().trim()
  return q ? files.value.filter(f => f.path.toLowerCase().includes(q)) : files.value
})

async function uploadFiles(ev: Event) {
  const input = ev.target as HTMLInputElement
  if (!input.files?.length) return
  busy.value = 'upload'
  try {
    for (const file of Array.from(input.files)) {
      await auth.upload(`/api/admin/builds/${props.buildId}/files`, 'file', file, { path: `mods/${file.name}` })
    }
    await load()
    emit('refresh')
  } finally {
    busy.value = null
    input.value = ''
  }
}

async function deleteFile(id: string) {
  if (!confirm('Delete file?')) return
  busy.value = `delete-${id}`
  try {
    await auth.request(`/api/admin/builds/${props.buildId}/files/${id}`, { method: 'DELETE' })
    await load()
    emit('refresh')
  } finally {
    busy.value = null
  }
}



watch(() => props.buildId, load)
</script>

<template>
  <section class="noro-panel overflow-hidden">
    <div class="flex items-center justify-between border-b border-[var(--noro-border)] px-5 py-4">
      <div class="flex items-center gap-3">
        <div class="flex size-10 items-center justify-center rounded-lg bg-black/20 text-[var(--noro-cream)]">
          <UIcon name="i-lucide-folder" class="size-6" />
        </div>
        <div class="font-bold text-[var(--noro-text)]">File Browser</div>
      </div>
      <div class="flex items-center gap-2">
        <input v-model="search" class="noro-input-sm w-48" placeholder="Search files..." />
        <label class="noro-btn noro-btn-secondary cursor-pointer">
          <UIcon name="i-lucide-upload" class="size-4" /> Upload
          <input type="file" multiple class="hidden" @change="uploadFiles" />
        </label>
        <AtomButton variant="dark" :loading="pending" @click="load">Refresh</AtomButton>
      </div>
    </div>

    <div class="flex border-b border-[var(--noro-border)] bg-black/10 px-4 py-1.5 text-[10px] uppercase tracking-widest text-[var(--noro-muted)]">
      <div class="flex-1">Path</div>
      <div class="w-32"></div>
    </div>

    <div class="noro-scroll max-h-[420px] overflow-auto text-sm">
      <div
        v-for="f in filtered"
        :key="f.id"
        class="group flex items-center border-b border-[var(--noro-border)] px-4 py-2 hover:bg-white/5"
      >
        <div class="flex-1 min-w-0 font-mono text-[var(--noro-text)] truncate">
          {{ f.path }}
        </div>

        <div class="w-32 flex justify-end gap-1 opacity-0 group-hover:opacity-100">
          <a :href="`/files/${f.sha1}`" download class="noro-btn noro-btn-dark !min-h-7 !text-[10px] !px-2">↓</a>
          <button class="noro-btn noro-btn-dark !min-h-7 !text-[10px] !px-2 text-[var(--noro-danger)]" @click.stop="deleteFile(f.id)">×</button>
        </div>
      </div>

      <div v-if="!filtered.length" class="p-8 text-center text-[var(--noro-muted)]">
        No files match.
      </div>
    </div>
  </section>
</template>
