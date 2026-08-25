<script setup lang="ts">
/**
 * Предпросмотр плашки роли — той же картинкой, что уедет в игру.
 *
 * Рисует её мастер, а не браузер, и это принципиально: нарисовать плашку здесь
 * заново значило бы завести вторую реализацию шрифта, и она разъехалась бы с
 * настоящей на первой же правке. Пусть медленнее, зато ровно то же самое.
 */
const props = defineProps<{
  /** Что написано в поле префикса. Пусто — покажем название роли. */
  text: string
  /** Название роли: запасная надпись, ровно как в игре. */
  fallback: string
  color: string
}>()

const auth = useAuth()
const { t } = useT()

const src = ref('')
const failed = ref(false)
let current = ''

/**
 * Картинка тянется запросом с токеном, а не тегом `<img src>`: маршрут закрыт
 * админской авторизацией, а заголовок к тегу не приложить.
 */
async function draw() {
  const text = (props.text || props.fallback || '').trim()
  const key = `${text}|${props.color}`
  if (key === current) return
  current = key
  try {
    const query = new URLSearchParams({ text, color: props.color })
    const blob = await auth.request<Blob>(`/api/admin/prefix-badge?${query}`, { responseType: 'blob' })
    if (key !== current) return
    if (src.value) URL.revokeObjectURL(src.value)
    src.value = URL.createObjectURL(blob)
    failed.value = false
  } catch {
    failed.value = true
  }
}

// Ждём, пока перестанут печатать: иначе запрос уходит на каждую букву.
let timer: ReturnType<typeof setTimeout>
watch(() => [props.text, props.fallback, props.color], () => {
  clearTimeout(timer)
  timer = setTimeout(draw, 250)
}, { immediate: true })

onBeforeUnmount(() => {
  clearTimeout(timer)
  if (src.value) URL.revokeObjectURL(src.value)
})
</script>

<template>
  <div class="grid gap-2">
    <span class="noro-label">{{ t('admin-role-badge-preview') }}</span>

    <!-- Фон под цвет чата: на светлой карточке плашка выглядела бы не так, как
         окажется в игре. -->
    <div class="flex items-center gap-3 rounded-[var(--noro-r-sm)] bg-[var(--noro-bg-deep)] p-4">
      <img
        v-if="src && !failed"
        :src="src"
        alt=""
        class="h-[28px] w-auto"
        style="image-rendering: pixelated"
      >
      <span v-else-if="failed" class="text-xs text-[var(--noro-magenta)]">
        {{ t('admin-role-badge-failed') }}
      </span>

      <!-- Ник шрифтом сайта, а не картинкой с мастера: шрифт здесь и так
           пиксельный, и рядом с плашкой смотрится как одна строка. Раньше стоял
           `font-mono` — вот он и выбивался, он не пиксельный. -->
      <span class="text-sm text-[var(--noro-text)]">Steve</span>
    </div>

    <span class="text-xs leading-5 text-[var(--noro-muted)]">
      {{ t('admin-role-badge-hint') }}
    </span>
  </div>
</template>
