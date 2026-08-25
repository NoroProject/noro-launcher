<script setup lang="ts">
/**
 * Типовые маски читов — в один клик.
 *
 * Пустой список бесполезен, а заполнять его руками начинают после первого
 * пойманного игрока, то есть поздно. Здесь то, что встречается на каждом
 * сервере: имена самих клиентов и слова, по которым узнаётся X-Ray.
 *
 * Маска, а не хэш: версий у каждого чита десятки, и хэш каждой не собрать.
 */
const emit = defineEmits<{ pick: [pattern: string, reason: string] }>()
const { t } = useT()

const PRESETS = [
  { pattern: '*xray*', reason: 'xray' },
  { pattern: '*wurst*', reason: 'client' },
  { pattern: '*meteor*', reason: 'client' },
  { pattern: '*impact*', reason: 'client' },
  { pattern: '*aristois*', reason: 'client' },
  { pattern: '*baritone*', reason: 'bot' },
  { pattern: '*killaura*', reason: 'combat' },
  { pattern: '*freecam*', reason: 'freecam' },
  { pattern: '*autoclick*', reason: 'macro' },
]
</script>

<template>
  <div class="flex flex-wrap items-center gap-2">
    <span class="noro-label">{{ t('admin-blocklist-presets') }}</span>
    <button
      v-for="preset in PRESETS"
      :key="preset.pattern"
      type="button"
      class="noro-chip px-2 py-1 text-xs font-bold text-[var(--noro-muted)] hover:text-[var(--noro-cream)]"
      :title="t(`admin-blocklist-preset-${preset.reason}`)"
      @click="emit('pick', preset.pattern, t(`admin-blocklist-preset-${preset.reason}`))"
    >{{ preset.pattern }}</button>
  </div>
</template>
