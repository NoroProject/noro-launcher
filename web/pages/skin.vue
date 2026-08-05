<script setup lang="ts">
import type { UserProfile } from '~/types/api'

const auth = useAuth()
await auth.loadMe()

const MAX_BYTES = 256 * 1024

const file = ref<File | null>(null)
const uploading = ref(false)
const message = ref<string | null>(null)
const error = ref<string | null>(null)
const dragging = ref(false)
const input = ref<HTMLInputElement | null>(null)

const hasSkin = computed(() => Boolean(auth.user.value?.skin_url))

/** Проверка до отправки: сетевой круг ради заведомо неподходящего файла не нужен. */
function accept(next: File | null) {
  error.value = null
  if (!next) {
    file.value = null
    return
  }
  if (next.type !== 'image/png') {
    error.value = 'Only PNG files are supported.'
    return
  }
  if (next.size > MAX_BYTES) {
    error.value = `File is too large — ${Math.round(next.size / 1024)} KB of 256 KB allowed.`
    return
  }
  file.value = next
  message.value = null
}

function onPick(event: Event) {
  accept((event.target as HTMLInputElement).files?.[0] || null)
}

function onDrop(event: DragEvent) {
  dragging.value = false
  accept(event.dataTransfer?.files?.[0] || null)
}

async function uploadSkin() {
  if (!file.value) return
  uploading.value = true
  error.value = null
  try {
    auth.user.value = await auth.upload<UserProfile>('/api/me/skin', 'skin', file.value)
    message.value = 'Skin updated'
    file.value = null
    if (input.value) input.value.value = ''
  } catch (err) {
    error.value = 'Upload failed. Try again.'
    console.error(err)
  } finally {
    uploading.value = false
  }
}

async function deleteSkin() {
  try {
    auth.user.value = await auth.request<UserProfile>('/api/me/skin', { method: 'DELETE' })
    file.value = null
    message.value = 'Skin deleted'
  } catch (err) {
    error.value = 'Could not delete the skin.'
    console.error(err)
  }
}
</script>

<template>
  <!--
    Загрузка переведена на зону перетаскивания: раньше стоял сырой input type=file
    с надписью «Choose File no file selected», из которой не следовало ни что
    класть, ни какого размера. Ограничения теперь проверяются до отправки.
    Карточки «PNG only» и «Instant profile» убраны — они повторяли текст выше.
  -->
  <NoroShell title="SKIN" subtitle="Your Minecraft appearance">
    <template #actions>
      <AtomButton variant="secondary" icon="i-lucide-arrow-left" to="/cabinet">Cabinet</AtomButton>
    </template>

    <div class="grid gap-4 xl:grid-cols-[360px_1fr]">
      <section class="noro-panel bg-[var(--noro-bg-deep)] p-6">
        <h2 class="noro-label mb-4">Preview</h2>
        <SkinPreview3D :skin-url="auth.user.value?.skin_url" :cape-url="auth.user.value?.cape_url" />
        <p class="mt-4 text-xs leading-5 text-[var(--noro-muted)]">
          Capes are assigned by admins and show up here automatically.
        </p>
      </section>

      <section class="noro-panel p-6">
        <h2 class="noro-label mb-2">Upload a skin</h2>
        <p class="mb-4 text-sm text-[var(--noro-muted)]">
          PNG in the standard Minecraft layout, up to 256&nbsp;KB.
        </p>

        <label
          class="grid cursor-pointer place-items-center gap-3 rounded-[var(--noro-r-sm)] border border-dashed px-6 py-10 text-center transition-colors duration-100"
          :class="dragging
            ? 'border-[var(--noro-cream)] bg-[color-mix(in_srgb,var(--noro-cream)_8%,var(--noro-input))]'
            : 'border-[var(--noro-border)] bg-[var(--noro-input)] hover:border-[var(--noro-muted)]'"
          @dragover.prevent="dragging = true"
          @dragleave.prevent="dragging = false"
          @drop.prevent="onDrop"
        >
          <UIcon name="i-lucide-upload" class="size-6 text-[var(--noro-muted)]" />
          <span class="text-sm text-[var(--noro-text)]">
            {{ file ? file.name : 'Drop a PNG here or click to choose' }}
          </span>
          <span v-if="file" class="noro-label">{{ Math.round(file.size / 1024) }} KB</span>
          <input ref="input" class="hidden" type="file" accept="image/png" @change="onPick">
        </label>

        <div class="mt-4 flex flex-wrap items-center gap-3">
          <AtomButton
            variant="primary"
            icon="i-lucide-upload"
            :disabled="!file || uploading"
            @click="uploadSkin"
          >
            {{ uploading ? 'Uploading…' : 'Upload' }}
          </AtomButton>
          <!-- Удаление отодвинуто от основной кнопки: соседство с Upload
               провоцировало промах по необратимому действию. -->
          <AtomButton
            v-if="hasSkin"
            variant="ghost"
            icon="i-lucide-trash-2"
            @click="deleteSkin"
            class="ml-auto text-[var(--noro-danger)]"
          >
            Remove current skin
          </AtomButton>
        </div>

        <UAlert
          v-if="error"
          class="mt-4"
          color="error"
          variant="subtle"
          icon="i-lucide-circle-alert"
          :description="error"
        />
        <UAlert
          v-else-if="message"
          class="mt-4"
          color="success"
          variant="subtle"
          icon="i-lucide-check"
          :description="message"
        />
      </section>
    </div>
  </NoroShell>
</template>
