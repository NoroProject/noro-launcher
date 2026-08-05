<script setup lang="ts">
import type { ServerRow } from '~/types/api'
import type { PermissionEntry } from '~/types/permissions'

const props = defineProps<{
  entry: PermissionEntry
  servers: ServerRow[]
  /** Пояснение из каталога; у игровых узлов его нет. */
  label?: string | null
  busy?: boolean
}>()

const emit = defineEmits<{
  remove: []
  move: [serverId: string | null]
}>()

function onContextChange(event: Event) {
  const next = (event.target as HTMLSelectElement).value || null
  if (next === props.entry.server_id) return
  emit('move', next)
}
</script>

<template>
  <div class="flex items-center gap-3 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] px-3 py-2">
    <div class="min-w-0 flex-1">
      <code class="block truncate font-mono text-xs text-[var(--noro-blue)]">{{ entry.permission }}</code>
      <span v-if="label" class="block truncate text-xs text-[var(--noro-muted)]">{{ label }}</span>
    </div>

    <!-- Узкий: тут выбирают из горстки сборок, а не пишут текст. -->
    <div class="w-32 shrink-0">
      <select
        class="noro-input noro-select py-1 text-xs"
        :disabled="busy"
        :value="entry.server_id || ''"
        @change="onContextChange"
      >
        <option value="">Global</option>
        <option v-for="server in servers" :key="server.id" :value="server.id">{{ server.name }}</option>
      </select>
    </div>

    <AtomButton variant="danger"
      :loading="busy"
      icon="i-lucide-x"
      size="sm"
      @click="emit('remove')"
    />
  </div>
</template>
