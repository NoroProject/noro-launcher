<script setup lang="ts">
import type { CapeRow } from '~/types/cape'

const auth = useAuth()

const notify = useNotify()
await auth.loadMe()

const name = ref('')
const file = ref<File | null>(null)
const busy = ref(false)
const message = ref<string | null>(null)
const { data: capes, refresh, pending, error } = await useAsyncData('admin-capes', () =>
  auth.request<CapeRow[]>('/api/admin/capes'), { default: () => [] }
)

const fileLabel = computed(() => file.value?.name || 'Choose PNG cape')

/** Локальный превью выбранного файла — видно, что грузишь, ещё до отправки. */
const preview = ref<string | null>(null)

function setFile(next: File | null) {
  if (preview.value) URL.revokeObjectURL(preview.value)
  file.value = next
  preview.value = next ? URL.createObjectURL(next) : null
}

function onFile(event: Event) {
  const input = event.target as HTMLInputElement
  setFile(input.files?.[0] || null)
}

onBeforeUnmount(() => {
  if (preview.value) URL.revokeObjectURL(preview.value)
})

async function uploadCape() {
  if (!file.value || !name.value.trim()) return
  busy.value = true
  message.value = null
  try {
    await auth.upload<CapeRow>('/api/admin/capes', 'cape', file.value, { name: name.value.trim() })
    name.value = ''
    setFile(null)
    message.value = 'Cape uploaded'
    await refresh()
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function deleteCape(cape: CapeRow) {
  busy.value = true
  try {
    await auth.request(`/api/admin/capes/${cape.id}`, { method: 'DELETE' })
    await refresh()
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <NoroShell title="CAPES" subtitle="Upload cape PNG files and assign them from user management">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        Refresh
      </AtomButton>
    </template>

    <UAlert v-if="error" class="mb-5" color="error" variant="subtle" icon="i-lucide-circle-alert" :description="humanError(error)" />
    <UAlert v-if="message" class="mb-5" color="success" variant="subtle" icon="i-lucide-check" :description="message" />

    <div class="grid gap-5 xl:grid-cols-[380px_1fr]">
      <section class="noro-panel p-5">
        <h2 class="text-2xl font-black text-[var(--noro-text)]">Upload Cape</h2>
        <p class="mt-2 text-sm font-medium text-[var(--noro-muted)]">Only admins can upload capes. Players receive capes from their user profile.</p>
        <div class="mt-5 grid gap-3">
          <input v-model="name" class="noro-input" placeholder="Cape name">
          <!-- Нативный file input показывает «Choose File no file selected»
               системным шрифтом и ломает вид панели, поэтому он скрыт. -->
          <label class="cursor-pointer rounded-lg border border-dashed border-[var(--noro-border)] bg-[var(--noro-input)] p-4 transition hover:border-[var(--noro-cream)]/50">
            <span class="noro-label">PNG file</span>
            <span class="mt-1 flex items-center gap-2">
              <UIcon :name="file ? 'i-lucide-file-check-2' : 'i-lucide-upload-cloud'" class="size-5 shrink-0 text-[var(--noro-cream)]" />
              <span class="min-w-0 truncate font-bold text-[var(--noro-cream)]">{{ fileLabel }}</span>
            </span>
            <span v-if="preview" class="mt-3 flex items-center gap-3">
              <CapePreview :url="preview" alt="Selected cape" class="w-12 shrink-0" />
              <span class="text-xs text-[var(--noro-muted)]">Front side preview</span>
            </span>
            <input class="hidden" type="file" accept="image/png" @change="onFile">
          </label>
          <AtomButton
            variant="primary"
            icon="i-lucide-upload"
            :disabled="busy || !file || !name.trim()"
            @click="uploadCape"
          >
            Upload Cape
          </AtomButton>
        </div>
      </section>

      <section class="grid gap-4 content-start sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
        <article
          v-for="cape in capes"
          :key="cape.id"
          class="group flex min-w-0 gap-4 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-panel)] p-4 transition hover:border-[var(--noro-cream)]/40"
        >
          <CapePreview :url="cape.url" :alt="cape.name" class="w-16 shrink-0" />
          <div class="flex min-w-0 flex-1 flex-col">
            <h3 class="truncate font-black text-[var(--noro-cream)]">{{ cape.name }}</h3>
            <p class="mt-1 text-xs font-bold uppercase tracking-wider text-[var(--noro-blue)]">{{ Math.ceil(cape.size / 1024) }} KB</p>
            <div class="mt-auto flex items-center gap-2 pt-3">
              <UTooltip text="Open full texture">
                <AtomButton icon="i-lucide-external-link" variant="ghost" size="sm" :to="cape.url" target="_blank" />
              </UTooltip>
              <UTooltip text="Delete cape">
                <AtomButton icon="i-lucide-trash-2" variant="ghost" size="sm" :disabled="busy" @click="deleteCape(cape)" />
              </UTooltip>
            </div>
          </div>
        </article>
        <EmptyState v-if="!capes?.length" icon="i-lucide-flag" title="No capes uploaded" class="sm:col-span-2 xl:col-span-3 2xl:col-span-4" />
      </section>
    </div>
  </NoroShell>
</template>
