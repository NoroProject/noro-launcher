<script setup lang="ts">
/**
 * Плашка роли картинкой — та же, что игрок видит в чате.
 *
 * Раньше на её месте стоял цветной кружок: он говорил только «роль какая-то
 * другая», а какая — приходилось читать рядом. Плашка отвечает на это сама и
 * совпадает с тем, что человек уже видел в игре.
 *
 * Рисует её мастер: вторая реализация шрифта в вебе разъехалась бы с первой на
 * первой же правке. Не нарисовалась — остаётся кружок, а не пустое место.
 */
const props = withDefaults(defineProps<{
  roleId: string
  color?: string | null
  /** Высота в пикселях. Плашка семипиксельная, поэтому кратно ей. */
  height?: number
}>(), { color: null, height: 14 })

const { masterUrl } = useApi()

const failed = ref(false)
// Полный адрес мастера, а не относительный путь: сайт живёт на своём порту, и
// относительная ссылка уходила бы к нему, а картинку отдаёт мастер.
const src = computed(() => `${masterUrl.value}/api/roles/${props.roleId}/badge.png`)
</script>

<template>
  <img
    v-if="!failed"
    :src="src"
    alt=""
    class="w-auto shrink-0"
    :style="{ height: `${height}px`, imageRendering: 'pixelated' }"
    @error="failed = true"
  >
  <span
    v-else
    class="size-3 shrink-0 rounded-[2px]"
    :style="{ backgroundColor: color || 'var(--noro-magenta)' }"
  />
</template>
