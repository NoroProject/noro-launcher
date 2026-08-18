<script setup lang="ts">
/**
 * Заголовок и текст записи на всех языках сразу, вкладками.
 * Список языков запрашивается с Мастера (источник правды — страница переводов).
 */
import type { LocalizedText } from '~/types/rules'

defineProps<{
  titleLabel: string
  titlePlaceholder: string
  textLabel: string
  textPlaceholder: string
  /** Подпись поля формулировки наказания. Не задана — поля нет (у разделов). */
  reasonLabel?: string
  reasonPlaceholder?: string
  reasonHint?: string
}>()

const auth = useAuth()
const items = defineModel<LocalizedText[]>({ required: true })
const active = ref(BASE_LOCALE)

const { data: remoteLocales } = await useAsyncData('locales-list', async () => {
  try {
    const list = await auth.request<{ locale: string }[]>('/api/launcher/locales')
    return list.map(l => l.locale)
  } catch {
    return ['ru', 'en']
  }
})

const localeList = computed(() => {
  const codes = new Set([BASE_LOCALE, ...(remoteLocales.value || []), ...items.value.map(i => i.locale)])
  return Array.from(codes).map(c => getLocaleOption(c))
})

const base = computed(() => items.value.find(i => i.locale === BASE_LOCALE))
const current = computed(() => items.value.find(i => i.locale === active.value))
const translated = (code: string) => {
  const item = items.value.find(i => i.locale === code)
  return !!item?.title.trim()
}

function ensureItem(code: string) {
  if (!items.value.some(i => i.locale === code)) {
    items.value.push({ locale: code, title: '', description: '', punish_reason: '' })
  }
}

function selectLocale(code: string) {
  ensureItem(code)
  active.value = code
}

function set(field: 'title' | 'description' | 'punish_reason', value: string) {
  ensureItem(active.value)
  items.value = items.value.map(item =>
    item.locale === active.value ? { ...item, [field]: value } : item,
  )
}
</script>

<template>
  <div class="grid gap-4">
    <div class="flex flex-wrap items-center gap-2">
      <AtomButton
        v-for="loc in localeList"
        :key="loc.code"
        :variant="active === loc.code ? 'primary' : 'secondary'"
        size="sm"
        @click="selectLocale(loc.code)"
      >
        <span>{{ loc.label }}</span>
        <span
          v-if="loc.code !== BASE_LOCALE"
          class="ml-2 inline-block size-2 rounded-full align-middle"
          :class="translated(loc.code) ? 'bg-[var(--noro-green)]' : 'bg-[var(--noro-border)]'"
        />
      </AtomButton>

      <span v-if="active !== BASE_LOCALE" class="noro-label noro-label-inline ml-auto">
        Empty = show the {{ localeLabel(BASE_LOCALE) }} text
      </span>
    </div>

    <label class="block">
      <span class="noro-label">{{ titleLabel }}</span>
      <input
        :value="current?.title"
        class="noro-input w-full"
        :placeholder="active === BASE_LOCALE ? titlePlaceholder : base?.title"
        @input="set('title', ($event.target as HTMLInputElement).value)"
      >
    </label>

    <label class="block">
      <span class="noro-label">{{ textLabel }}</span>
      <textarea
        :value="current?.description"
        rows="4"
        class="noro-input w-full resize-y"
        :placeholder="active === BASE_LOCALE ? textPlaceholder : base?.description"
        @input="set('description', ($event.target as HTMLTextAreaElement).value)"
      />
    </label>

    <label v-if="reasonLabel" class="block">
      <span class="noro-label">{{ reasonLabel }}</span>
      <input
        :value="current?.punish_reason"
        class="noro-input w-full"
        :placeholder="active === BASE_LOCALE ? reasonPlaceholder : base?.punish_reason"
        @input="set('punish_reason', ($event.target as HTMLInputElement).value)"
      >
      <span v-if="reasonHint" class="mt-2 block text-xs text-[var(--noro-muted)]">{{ reasonHint }}</span>
    </label>
  </div>
</template>
