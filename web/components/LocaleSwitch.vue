<script setup lang="ts">
/**
 * Выпадающий переключатель языка без флагов.
 */
const auth = useAuth()
const { locale, setLocale } = useT()

const { data: remoteLocales } = await useAsyncData('available-locales', async () => {
  try {
    const list = await auth.request<{ locale: string }[]>('/api/launcher/locales')
    if (list && list.length) {
      return list.map(l => l.locale)
    }
  } catch {
    // fallback
  }
  return ['ru', 'en']
})

const options = computed(() => {
  const codes = new Set(remoteLocales.value || ['ru', 'en'])
  codes.add(locale.value)
  return Array.from(codes).map(code => getLocaleOption(code))
})

const isOpen = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

function select(code: string) {
  setLocale(code)
  isOpen.value = false
}

const currentOpt = computed(() => getLocaleOption(locale.value))

function onClickOutside(e: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(e.target as Node)) {
    isOpen.value = false
  }
}

onMounted(() => window.addEventListener('click', onClickOutside))
onUnmounted(() => window.removeEventListener('click', onClickOutside))
</script>

<template>
  <div ref="dropdownRef" class="relative inline-block text-left">
    <button
      type="button"
      class="flex h-9 items-center gap-2 rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] px-3 text-xs font-bold text-[var(--noro-text)] border border-[var(--noro-border)] hover:border-[var(--noro-border-hover)] transition cursor-pointer"
      @click.stop="isOpen = !isOpen"
    >
      <UIcon name="i-lucide-globe" class="size-4 text-[var(--noro-blue)]" />
      <span>{{ currentOpt.short }}</span>
      <UIcon name="i-lucide-chevron-down" class="size-3 text-[var(--noro-muted)] transition-transform duration-200" :class="{ 'rotate-180': isOpen }" />
    </button>

    <div
      v-if="isOpen"
      class="absolute right-0 z-50 mt-1 min-w-[140px] rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg)] p-1 shadow-xl backdrop-blur-md"
    >
      <button
        v-for="loc in options"
        :key="loc.code"
        type="button"
        class="flex w-full items-center justify-between gap-3 rounded-md px-3 py-2 text-xs font-medium transition text-left cursor-pointer"
        :class="locale === loc.code
          ? 'bg-[var(--noro-panel-2)] text-[var(--noro-cream)] font-bold'
          : 'text-[var(--noro-text)] hover:bg-[var(--noro-input)]'"
        @click="select(loc.code)"
      >
        <span>{{ loc.label }}</span>
        <UIcon v-if="locale === loc.code" name="i-lucide-check" class="size-3.5 text-[var(--noro-cream)] shrink-0" />
      </button>
    </div>
  </div>
</template>
