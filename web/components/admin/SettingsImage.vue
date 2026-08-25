<script setup lang="ts">
/**
 * Одна иллюстрация инстанса: превью, загрузка файлом и адрес.
 *
 * Превью лежит на шахматке — прозрачный фон иначе не отличить от тёмной панели,
 * а от прозрачности зависит вид картинки на сайте: у неё нет подложки с рамкой.
 */

import type { SettingItem } from '~/types/settings'

const props = defineProps<{
  item: SettingItem
  title: string
  desc: string
  transparent: boolean
  canEdit: boolean
}>()

const url = defineModel<string>({ required: true })
const emit = defineEmits<{ uploaded: [] }>()

const auth = useAuth()
const notify = useNotify()
const { t } = useT()

const input = ref<HTMLInputElement | null>(null)
const uploading = ref(false)

async function upload(event: Event) {
  const el = event.target as HTMLInputElement
  const file = el.files?.[0]
  if (!file) return

  uploading.value = true
  try {
    const body = new FormData()
    body.append('image', file)
    const res = await auth.request<{ url: string }>(
      `/api/admin/settings/image/${props.item.key}`,
      { method: 'POST', body },
    )
    url.value = res.url
    notify.ok()
    emit('uploaded')
  } catch (e) {
    notify.fail(e)
  } finally {
    uploading.value = false
    el.value = ''
  }
}
</script>

<template>
  <div class="rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-panel-2)] p-4">
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div class="min-w-0">
        <h3 class="text-sm font-bold uppercase text-[var(--noro-cream)]">{{ title }}</h3>
        <p class="text-xs text-[var(--noro-muted)]">{{ desc }}</p>
      </div>
      <!-- GIF, WebP и PNG сохраняются как есть: анимация и прозрачность
           переживают только сами себя. -->
      <input
        ref="input"
        type="file"
        accept="image/png,image/gif,image/webp,image/jpeg"
        class="hidden"
        @change="upload"
      >
      <AtomButton
        v-if="canEdit"
        icon="i-lucide-upload"
        variant="secondary"
        size="sm"
        :loading="uploading"
        @click="input?.click()"
      >
        {{ t('admin-settings-hero-upload') }}
      </AtomButton>
    </div>

    <div v-if="url" class="mt-3 flex flex-wrap items-center gap-4">
      <div class="noro-checkerboard rounded-lg border border-[var(--noro-border)] p-1">
        <img :src="url" alt="" class="h-20 w-32 object-contain">
      </div>
      <span
        class="inline-block rounded px-1.5 py-0.5 text-[10px] font-black uppercase tracking-wider"
        :class="transparent
          ? 'bg-[color-mix(in_srgb,var(--noro-blue)_18%,transparent)] text-[var(--noro-blue)]'
          : 'bg-[var(--noro-panel)] text-[var(--noro-muted)]'"
      >
        {{ transparent ? t('admin-settings-image-transparent') : t('admin-settings-image-opaque') }}
      </span>
    </div>

    <!-- Тот же адрес полем: картинку можно не заливать, а взять по ссылке. -->
    <SettingsField v-model="url" :item="item" class="mt-3" />
  </div>
</template>
