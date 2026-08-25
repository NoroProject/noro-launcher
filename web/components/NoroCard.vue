<script setup lang="ts">
/**
 * Карточка раздела — одна на весь кабинет и админку.
 *
 * До неё каждая страница собирала шапку сама: где-то был значок, где-то нет,
 * заголовок то кремовый, то белый, то заглавными. На отдельной странице это
 * незаметно, а при переходе между ними выглядит как разные продукты.
 *
 * Значок обязателен не для красоты: он якорь, по которому раздел находят
 * взглядом, не читая заголовок. Раздел без своего значка — повод подумать, не
 * лишний ли он, а не повод рисовать шапку иначе.
 *
 * Заголовок обычным цветом текста, а не кремовым: кремовый — цвет главного
 * действия, и когда им же выкрашены все заголовки подряд, страница пестрит, а
 * кнопка перестаёт выделяться. Акцент в шапке один — значок.
 */
withDefaults(defineProps<{
  title: string
  /** Иконка `i-lucide-*`. */
  icon: string
  /** Одна строка о том, что здесь и зачем. */
  subtitle?: string
  /** Убрать внутренние отступы — для таблиц во всю ширину карточки. */
  flush?: boolean
}>(), { flush: false })
</script>

<template>
  <section class="noro-panel" :class="flush ? 'p-0' : 'p-5'">
    <header
      class="flex flex-wrap items-start justify-between gap-3"
      :class="flush ? 'p-5 pb-0' : ''"
    >
      <div class="min-w-0">
        <h2 class="flex items-center gap-2 text-sm font-bold text-[var(--noro-text)]">
          <UIcon :name="icon" class="size-4 shrink-0 text-[var(--noro-blue)]" />
          <span class="truncate">{{ title }}</span>
        </h2>
        <p v-if="subtitle" class="mt-1 text-xs leading-5 text-[var(--noro-muted)]">
          {{ subtitle }}
        </p>
      </div>

      <!-- Действия раздела всегда справа в шапке: кнопка «обновить», счётчик,
           фильтр. Внизу карточки их искали бы каждый раз заново. -->
      <div v-if="$slots.actions" class="flex shrink-0 items-center gap-2">
        <slot name="actions" />
      </div>
    </header>

    <div :class="flush ? '' : 'mt-4'">
      <slot />
    </div>
  </section>
</template>
