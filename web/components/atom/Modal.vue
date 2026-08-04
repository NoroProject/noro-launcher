<script setup lang="ts">
defineProps<{
  modelValue: boolean
  title: string
  subtitle?: string
  wide?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

function close() {
  emit('update:modelValue', false)
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="fixed inset-0 z-50 grid place-items-center overflow-y-auto bg-[var(--noro-bg-deep)]/90 p-4"
      @click.self="close"
    >
      <section
        class="noro-panel my-8 w-full overflow-visible"
        :class="wide ? 'max-w-3xl' : 'max-w-xl'"
      >
        <header class="flex items-start justify-between gap-4 bg-[var(--noro-bg-deep)] p-5">
          <div class="min-w-0">
            <h2 class="noro-pixel truncate text-xl uppercase text-[var(--noro-cream)]">{{ title }}</h2>
            <p v-if="subtitle" class="mt-2 text-sm font-semibold uppercase tracking-wider text-[var(--noro-muted)]">{{ subtitle }}</p>
          </div>
          <AtomButton icon="i-lucide-x" variant="ghost" size="sm" aria-label="Close" @click="close" />
        </header>
        <div class="overflow-visible p-5">
          <slot />
        </div>
      </section>
    </div>
  </Teleport>
</template>
