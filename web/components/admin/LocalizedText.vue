<script setup lang="ts">
/**
 * Заголовок и текст записи на всех языках сразу, вкладками.
 *
 * Вкладка базового языка правит саму запись, остальные — её переводы. Отсюда
 * и разное отношение к пустоте: без базового текста записи нет, а пустой
 * перевод просто означает «показывать исходный».
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

const items = defineModel<LocalizedText[]>({ required: true })
const active = ref(BASE_LOCALE)

const base = computed(() => items.value.find(i => i.locale === BASE_LOCALE))
const current = computed(() => items.value.find(i => i.locale === active.value))
const translated = (code: string) => {
  const item = items.value.find(i => i.locale === code)
  return !!item?.title.trim()
}

/**
 * Замена всего массива, а не правка поля на месте: элементы приходят через
 * модель, и мутация вглубь не всегда доходит до родителя.
 */
function set(field: 'title' | 'description' | 'punish_reason', value: string) {
  items.value = items.value.map(item =>
    item.locale === active.value ? { ...item, [field]: value } : item,
  )
}
</script>

<template>
  <div class="grid gap-4">
    <div class="flex flex-wrap items-center gap-2">
      <AtomButton
        v-for="loc in LOCALES"
        :key="loc.code"
        :variant="active === loc.code ? 'primary' : 'secondary'"
        size="sm"
        @click="active = loc.code"
      >
        {{ loc.label }}
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
