<script setup lang="ts">
import type { ServerRow } from '~/types/api'

const { t } = useT()

const props = defineProps<{
  permission: string
  /** Контексты, в которых право выдано. `null` — на всех сборках. */
  contexts: (string | null)[]
  servers: ServerRow[]
  /** Пояснение из каталога; у игровых узлов его нет. */
  label?: string | null
  busy?: boolean
}>()

const emit = defineEmits<{
  /** Включить или выключить один контекст — что именно, решает состояние чипа. */
  toggle: [serverId: string | null]
  remove: []
}>()

const global = computed(() => props.contexts.includes(null))

function on(serverId: string | null) {
  return props.contexts.includes(serverId)
}
</script>

<template>
  <div class="rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] px-3 py-2">
    <div class="flex items-center gap-3">
      <div class="min-w-0 flex-1">
        <code class="block truncate font-mono text-xs text-[var(--noro-blue)]">{{ permission }}</code>
        <span v-if="label" class="block truncate text-xs text-[var(--noro-muted)]">{{ t(label) }}</span>
      </div>
      <AtomButton variant="danger" :loading="busy" icon="i-lucide-x" size="sm" @click="emit('remove')" />
    </div>

    <!-- Чипы, а не select: право живёт сразу в нескольких контекстах, и список
         с одним выбранным значением такое состояние показать не может. -->
    <div class="mt-2 flex flex-wrap gap-1.5">
      <button
        type="button"
        class="noro-chip px-2 py-1 text-xs"
        :class="global ? 'noro-chip-on' : ''"
        :disabled="busy"
        @click="emit('toggle', null)"
      >
        All builds
      </button>
      <button
        v-for="server in servers"
        :key="server.id"
        type="button"
        class="noro-chip px-2 py-1 text-xs"
        :class="on(server.id) ? 'noro-chip-on' : ''"
        :disabled="busy || global"
        :title="global ? 'Already granted on every build' : undefined"
        @click="emit('toggle', server.id)"
      >
        {{ server.name }}
      </button>
    </div>
  </div>
</template>
