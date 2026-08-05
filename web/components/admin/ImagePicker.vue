<script setup lang="ts">
/**
 * Загрузка картинки файлом вместо вставки ссылки.
 *
 * Ссылка требовала где-то отдельно разместить изображение и переживала бы
 * смерть чужого хостинга. Файл уходит в content-addressed стор мастера и
 * возвращается иммутабельным URL, который уже нельзя сломать снаружи.
 */
const props = defineProps<{
  modelValue: string
  label?: string
  endpoint?: string
}>()

const emit = defineEmits<{ 'update:modelValue': [string] }>()

const auth = useAuth()
const uploading = ref(false)
const error = ref<string | null>(null)
const dragging = ref(false)
const input = ref<HTMLInputElement | null>(null)

/**
 * `<label for>` вместо обёртки: input лежит рядом, а не внутри зоны, иначе
 * `@drop` на label ловил бы и клик по самому input. Id обязан быть уникальным —
 * на странице таких пикеров может оказаться несколько.
 */
const inputId = useId()

const MAX_BYTES = 4 * 1024 * 1024

async function send(file: File | null | undefined) {
  error.value = null
  if (!file) return
  if (!file.type.startsWith('image/')) {
    error.value = 'Only image files are supported.'
    return
  }
  if (file.size > MAX_BYTES) {
    error.value = `File is too large — ${Math.round(file.size / 1024 / 1024)} MB of 4 MB allowed.`
    return
  }
  uploading.value = true
  try {
    const res = await auth.upload<{ url: string }>(
      props.endpoint || '/api/admin/news/image',
      'image',
      file
    )
    emit('update:modelValue', res.url)
  } catch (err) {
    error.value = 'Upload failed. Try again.'
    console.error(err)
  } finally {
    uploading.value = false
    if (input.value) input.value.value = ''
  }
}
</script>

<template>
  <div class="grid gap-2">
    <span class="noro-label">{{ label || 'Preview image' }}</span>

    <div v-if="modelValue" class="grid gap-2">
      <img :src="modelValue" alt="" class="aspect-video w-full rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] object-cover">
      <div class="flex flex-wrap gap-2">
        <AtomButton
          variant="secondary"
          icon="i-lucide-refresh-cw"
          size="sm"
          @click="input?.click()"
        >
          Replace
        </AtomButton>
        <AtomButton
          variant="ghost"
          icon="i-lucide-x"
          size="sm"
          @click="emit('update:modelValue', '')"
          class="text-[var(--noro-danger)]"
        >
          Remove
        </AtomButton>
      </div>
    </div>

    <label
      v-else
      :for="inputId"
      class="grid cursor-pointer place-items-center gap-2 rounded-[var(--noro-r-sm)] border border-dashed px-4 py-8 text-center transition-colors duration-100"
      :class="dragging
        ? 'border-[var(--noro-cream)] bg-[color-mix(in_srgb,var(--noro-cream)_8%,var(--noro-input))]'
        : 'border-[var(--noro-border)] bg-[var(--noro-input)] hover:border-[var(--noro-muted)]'"
      @dragover.prevent="dragging = true"
      @dragleave.prevent="dragging = false"
      @drop.prevent="dragging = false; send($event.dataTransfer?.files?.[0])"
    >
      <UIcon name="i-lucide-image-plus" class="size-5 text-[var(--noro-muted)]" />
      <span class="text-sm text-[var(--noro-text)]">
        {{ uploading ? 'Uploading…' : 'Drop an image or click to choose' }}
      </span>
      <span class="text-xs text-[var(--noro-muted)]">PNG or JPEG, up to 4 MB</span>
    </label>

    <input :id="inputId" ref="input" class="hidden" type="file" accept="image/*" @change="send(($event.target as HTMLInputElement).files?.[0])">

    <p v-if="error" class="text-xs text-[var(--noro-danger)]">{{ error }}</p>
  </div>
</template>
