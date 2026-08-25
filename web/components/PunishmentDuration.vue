<script setup lang="ts">
/** Срок наказания: поле, разбор введённого и быстрые пресеты. */
const duration = defineModel<string>({ required: true })

const { t } = useT()
const PRESETS = ['1h', '12h', '1d', '7d', '30d', 'forever']

/** `null` — навсегда, `NaN` — в поле опечатка. */
const minutes = computed(() => parseDuration(duration.value))
const broken = computed(() => Number.isNaN(minutes.value))

/**
 * Как поняли срок — в самой подписи поля, одной строкой без переноса.
 * Отдельной строкой под полем текст менял высоту панели на каждое нажатие.
 */
const hint = computed(() => {
  if (!duration.value.trim()) return t('admin-punish-duration-hint-empty')
  if (broken.value) return 'try 7d, 12h, 1d 6h, 30m'
  const until = new Date(Date.now() + minutes.value! * 60_000)
  return t('admin-punish-duration-hint-until', { duration: formatDuration(minutes.value!), until: until.toLocaleDateString() })
})
</script>

<template>
  <div class="grid min-w-0 gap-3">
    <label class="block">
      <span class="noro-label mb-2 flex items-baseline gap-2">
        {{ t('admin-punish-duration-label') }}
        <span
          class="truncate whitespace-nowrap normal-case tracking-normal"
          :class="broken ? 'text-[var(--noro-magenta)]' : 'text-[var(--noro-muted)]'"
        >{{ hint }}</span>
      </span>
      <input
        v-model="duration"
        class="noro-input w-full"
        :class="broken ? '!border-[var(--noro-magenta)]' : ''"
        placeholder="7d · 12h · 1d 6h · 30m"
      >
    </label>

    <div class="flex flex-wrap items-center gap-2">
      <button
        v-for="preset in PRESETS"
        :key="preset"
        type="button"
        class="noro-chip px-2 py-1 text-xs font-bold"
        :class="duration === preset || (preset === 'forever' && !duration) ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
        @click="duration = preset === 'forever' ? '' : preset"
      >{{ preset === 'forever' ? t('punishment-forever') : preset }}</button>
    </div>
  </div>
</template>
