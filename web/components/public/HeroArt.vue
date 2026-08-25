<script setup lang="ts">
/**
 * Иллюстрация главной.
 *
 * Подложка с рамкой нужна непрозрачной картинке — скриншот или рендер кадром
 * без неё торчит из тёмной страницы обрезанным прямоугольником. У вырезанного
 * рендера с альфа-каналом обводить нечего: рамка рисует границу там, где у
 * картинки её нет, поэтому он ложится прямо на фон, а объём ему даёт тень по
 * силуэту (`drop-shadow` считает её по альфе, `shadow` — по рамке блока).
 */
defineProps<{ src: string; transparent: boolean }>()
</script>

<template>
  <div class="group relative flex items-center justify-center">
    <!-- Ambient Glow Backing -->
    <div
      v-if="!transparent"
      class="absolute inset-0 rounded-3xl bg-gradient-to-tr from-[var(--noro-blue)]/20 via-transparent to-[var(--noro-cream)]/20 blur-2xl transition-all duration-500 group-hover:scale-105"
    />

    <!-- Render Container -->
    <div
      class="relative transition-all duration-500"
      :class="transparent ? '' : 'overflow-hidden rounded-3xl border border-[var(--noro-border)] bg-[var(--noro-panel)] p-2 shadow-2xl group-hover:border-[var(--noro-cream)]'"
    >
      <img
        :src="src"
        alt="Minecraft 3D Character Render"
        class="h-auto w-full transition-transform duration-700 group-hover:scale-[1.02]"
        :class="transparent ? 'object-contain drop-shadow-2xl' : 'rounded-2xl object-cover'"
      >
    </div>
  </div>
</template>
