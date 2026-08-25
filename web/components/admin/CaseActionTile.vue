<script setup lang="ts">
/**
 * Плитка действия разбора.
 *
 * Кнопка со значком не говорит, что случится: «срез чата» — это спросить у
 * сервера и подождать, а «заморозить» — прямо сейчас обездвижить человека.
 * Плитка несёт подпись под названием и гаснет, когда действие невозможно, —
 * так модератор видит цену нажатия до того, как нажал.
 */
const props = defineProps<{
  icon: string
  label: string
  hint?: string
  /** Действие меняет что-то в игре немедленно — плитка предупреждает цветом. */
  loud?: boolean
  busy?: boolean
  disabled?: boolean
}>()
defineEmits<{ click: [] }>()

const inactive = computed(() => props.disabled || props.busy)
</script>

<template>
  <button
    type="button"
    :disabled="inactive"
    class="group grid gap-1 border p-3 text-left transition"
    :class="[
      loud
        ? 'border-[var(--noro-danger)]/40 bg-[var(--noro-danger)]/5 hover:border-[var(--noro-danger)]'
        : 'border-[var(--noro-border)] bg-[var(--noro-panel)] hover:border-[var(--noro-blue)]',
      inactive ? 'cursor-not-allowed opacity-40' : 'cursor-pointer',
    ]"
    @click="$emit('click')"
  >
    <span class="flex items-center gap-2">
      <UIcon
        :name="busy ? 'i-lucide-loader-circle' : icon"
        class="size-4 shrink-0"
        :class="[
          busy && 'animate-spin',
          loud ? 'text-[var(--noro-danger)]' : 'text-[var(--noro-blue)]',
        ]"
      />
      <span class="truncate text-sm font-bold uppercase text-white">{{ label }}</span>
    </span>
    <span v-if="hint" class="text-xs leading-5 text-[var(--noro-muted)]">{{ hint }}</span>
  </button>
</template>
