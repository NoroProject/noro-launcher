<script setup lang="ts">
/**
 * Модалка на примитивах админки.
 *
 * Не `UModal`: тот приходит со своей светлой темой Nuxt UI, а весь остальной
 * интерфейс тёмный — и окно выпадало белым пятном.
 */
defineProps<{ title: string }>()
const open = defineModel<boolean>({ required: true })

/** Escape закрывает — иначе окно без явной кнопки становится ловушкой. */
function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') open.value = false
}

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 grid place-items-center bg-black/70 p-4"
      @click.self="open = false"
    >
      <div class="noro-panel w-full max-w-2xl overflow-hidden">
        <header class="flex items-center justify-between gap-4 border-b border-[var(--noro-border)] px-6 py-4">
          <h2 class="text-lg font-black text-[var(--noro-text)]">{{ title }}</h2>
          <button
            class="rounded p-1 text-[var(--noro-muted)] transition hover:bg-[var(--noro-input)] hover:text-[var(--noro-text)]"
            aria-label="Close"
            @click="open = false"
          >
            <UIcon name="i-lucide-x" class="size-5" />
          </button>
        </header>

        <div class="noro-scroll max-h-[70vh] overflow-y-auto p-6">
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>
