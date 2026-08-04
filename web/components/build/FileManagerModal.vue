<script setup lang="ts">
const open = defineModel<boolean>({ required: true })
const props = defineProps<{ buildId: string }>()
const emit = defineEmits<{ changed: [] }>()

function close() { open.value = false }
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 grid place-items-center overflow-y-auto bg-[var(--noro-bg-deep)]/90 p-4"
      @click.self="close"
    >
      <section class="noro-panel my-4 w-full max-w-4xl overflow-hidden">
        <header class="flex items-center justify-between bg-[var(--noro-bg-deep)] px-5 py-3">
          <div>
            <h2 class="noro-pixel text-lg uppercase text-[var(--noro-cream)]">File Manager</h2>
            <p class="text-xs font-semibold uppercase tracking-wider text-[var(--noro-muted)]">
              Build file browser
            </p>
          </div>
          <AtomButton icon="i-lucide-x" variant="ghost" size="sm" @click="close" />
        </header>
        <FileManagerCore :build-id="buildId" @changed="emit('changed')" />
      </section>
    </div>
  </Teleport>
</template>
