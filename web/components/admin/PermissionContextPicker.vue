<script setup lang="ts">
import type { ServerRow } from '~/types/api'

defineProps<{ servers: ServerRow[] }>()

const global = defineModel<boolean>('global', { required: true })
const picked = defineModel<string[]>('picked', { required: true })
const { t } = useT()

function toggle(serverId: string) {
  global.value = false
  picked.value = picked.value.includes(serverId)
    ? picked.value.filter(id => id !== serverId)
    : [...picked.value, serverId]
}
</script>

<template>
  <div>
    <span class="noro-label">{{ t('admin-perm-context') }}</span>
    <div class="mt-1 flex flex-wrap gap-2">
      <button
        type="button"
        class="noro-chip px-3 py-2 text-xs"
        :class="global ? 'noro-chip-on' : ''"
        @click="global = true"
      >
        {{ t('admin-perm-all-builds') }}
      </button>
      <button
        v-for="server in servers"
        :key="server.id"
        type="button"
        class="noro-chip px-3 py-2 text-xs"
        :class="!global && picked.includes(server.id) ? 'noro-chip-on' : ''"
        @click="toggle(server.id)"
      >
        {{ server.name }}
      </button>
    </div>
  </div>
</template>
