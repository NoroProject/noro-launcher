<script setup lang="ts">
import type { CapeRow } from '~/types/cape'

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
const notify = useNotify()
await auth.loadMe()

const name = ref('')
const file = ref<File | null>(null)
const busy = ref(false)
const message = ref<string | null>(null)
const error = ref<string | null>(null)
const dragging = ref(false)
const confirmDeleteId = ref<string | null>(null)

const { data: capes, refresh, pending } = await useAsyncData('admin-capes', () =>
  auth.requestList<CapeRow>('/api/admin/capes'), { default: () => [] }
)


const preview = ref<string | null>(null)

function setFile(next: File | null) {
  error.value = null
  if (preview.value) URL.revokeObjectURL(preview.value)
  if (!next) {
    file.value = null
    preview.value = null
    return
  }
  if (next.type !== 'image/png') {
    error.value = 'Only PNG cape images are supported.'
    return
  }
  if (next.size > 512 * 1024) {
    error.value = `File is too large — ${Math.round(next.size / 1024)} KB of 512 KB allowed.`
    return
  }
  file.value = next
  preview.value = URL.createObjectURL(next)
  if (!name.value.trim()) {
    name.value = next.name.replace(/\.[^/.]+$/, '')
  }
}

function onFile(event: Event) {
  const input = event.target as HTMLInputElement
  setFile(input.files?.[0] || null)
}

function onDrop(event: DragEvent) {
  dragging.value = false
  setFile(event.dataTransfer?.files?.[0] || null)
}

onBeforeUnmount(() => {
  if (preview.value) URL.revokeObjectURL(preview.value)
})

async function uploadCape() {
  if (!file.value || !name.value.trim()) return
  busy.value = true
  message.value = null
  error.value = null
  try {
    await auth.upload<CapeRow>('/api/admin/capes', 'cape', file.value, { name: name.value.trim() })
    name.value = ''
    setFile(null)
    message.value = 'Cape uploaded successfully'
    await refresh()
    notify.ok()
  } catch (e) {
    error.value = humanError(e)
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function deleteCape(id: string) {
  busy.value = true
  confirmDeleteId.value = null
  try {
    await auth.request(`/api/admin/capes/${id}`, { method: 'DELETE' })
    message.value = 'Cape deleted'
    await refresh()
    notify.ok()
  } catch (e) {
    error.value = humanError(e)
    notify.fail(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <NoroShell :title="t('admin-capes-title')" :subtitle="t('admin-capes-subtitle')">
    <template #actions>
      <AtomButton
        v-if="can('noro.admin.capes.edit')"
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
    </template>

    <UAlert v-if="error" class="mb-5" color="error" variant="subtle" icon="i-lucide-circle-alert" :description="error" />
    <UAlert v-if="message" class="mb-5" color="success" variant="subtle" icon="i-lucide-check" :description="message" />

    <div class="grid gap-6 xl:grid-cols-[340px_1fr]">
      <!-- Upload Panel -->
      <section class="noro-panel flex flex-col p-6">
        <h2 class="text-xl font-black text-[var(--noro-text)]">{{ t('admin-capes-add-title') }}</h2>
        <p class="mt-1 text-xs text-[var(--noro-muted)]">
          {{ t('admin-capes-add-subtitle') }}
        </p>

        <form class="mt-5 grid gap-4" @submit.prevent="uploadCape">
          <label class="block">
            <span class="noro-label mb-1.5 block">{{ t('admin-capes-name-label') }}</span>
            <input
              v-model="name"
              class="noro-input w-full"
              :placeholder="t('admin-capes-name-placeholder')"
              maxlength="48"
            >
          </label>

          <!-- Dropzone -->
          <label
            class="group relative grid cursor-pointer place-items-center gap-3 rounded-[var(--noro-r-sm)] border-2 border-dashed p-6 text-center transition-all duration-150"
            :class="dragging
              ? 'border-[var(--noro-cream)] bg-[color-mix(in_srgb,var(--noro-cream)_10%,var(--noro-input))]'
              : 'border-[var(--noro-border)] bg-[var(--noro-input)] hover:border-[var(--noro-cream)]/50'"
            @dragover.prevent="dragging = true"
            @dragleave.prevent="dragging = false"
            @drop.prevent="onDrop"
          >
            <div v-if="!preview" class="grid place-items-center gap-2">
              <UIcon name="i-lucide-upload-cloud" class="size-8 text-[var(--noro-cream)] transition-transform group-hover:scale-110" />
              <span class="text-xs font-bold text-[var(--noro-text)]">
                {{ file ? file.name : t('admin-capes-dropzone') }}
              </span>
              <span class="text-[10px] text-[var(--noro-muted)]">{{ t('admin-capes-dropzone-hint') }}</span>
            </div>

            <div v-else class="flex flex-col items-center gap-2">
              <div class="relative grid place-items-center rounded-md bg-[var(--noro-bg-deep)] p-3 border border-[var(--noro-border)]">
                <CapePreview :url="preview" alt="Preview" class="w-16 shadow-lg" />
              </div>
              <span class="truncate text-xs font-bold text-[var(--noro-cream)]">{{ file?.name }}</span>
            </div>

            <input class="hidden" type="file" accept="image/png" @change="onFile">
          </label>

          <AtomButton
            v-if="can('noro.admin.capes.edit')"
            variant="primary"
            icon="i-lucide-upload"
            type="submit"
            :disabled="busy || !file || !name.trim()"
            class="mt-2 w-full justify-center"
          >
            {{ busy ? t('admin-capes-uploading') : t('admin-capes-upload-btn') }}
          </AtomButton>
        </form>
      </section>

      <!-- Cape Catalog Minimal Grid -->
      <section class="noro-panel p-6">
        <h2 class="noro-label mb-4">{{ t('admin-capes-grid-title') }}</h2>

        <div v-if="capes?.length" class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
          <UTooltip
            v-for="cape in capes"
            :key="cape.id"
            :text="`${cape.name} (${Math.ceil(cape.size / 1024)} KB)`"
          >
            <article
              class="group relative flex aspect-[10/16] cursor-pointer flex-col overflow-hidden rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] transition duration-200 hover:scale-105 hover:border-[var(--noro-cream)] hover:shadow-xl"
            >
              <!-- Cape 2D Render -->
              <div class="relative flex size-full items-center justify-center p-1.5">
                <CapePreview :url="cape.url" :alt="cape.name" class="h-full w-auto max-w-full shadow-md" />
              </div>

              <!-- Hover Overlay with Actions -->
              <div class="absolute inset-0 flex flex-col justify-between bg-black/80 p-2 opacity-0 backdrop-blur-xs transition-opacity duration-150 group-hover:opacity-100">
                <p class="truncate text-[10px] font-bold text-[var(--noro-cream)]">{{ cape.name }}</p>

                <div class="flex items-center justify-between">
                  <a
                    :href="cape.url"
                    target="_blank"
                    class="grid size-6 place-items-center rounded bg-white/10 text-white transition hover:bg-white/20"
                    title="Open PNG"
                    @click.stop
                  >
                    <UIcon name="i-lucide-external-link" class="size-3.5" />
                  </a>

                  <button
                    class="grid size-6 place-items-center rounded bg-red-500/20 text-red-400 transition hover:bg-red-500/40 hover:text-white"
                    title="Delete cape"
                    @click.stop="deleteCape(cape.id)"
                  >
                    <UIcon name="i-lucide-trash-2" class="size-3.5" />
                  </button>
                </div>
              </div>
            </article>
          </UTooltip>
        </div>

        <EmptyState
          v-else
          icon="i-lucide-flag"
          :title="t('admin-capes-empty-title')"
          :text="t('admin-capes-empty-text')"
          class="min-h-[280px]"
        />
      </section>
    </div>
  </NoroShell>
</template>
