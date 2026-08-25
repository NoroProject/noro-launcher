<script setup lang="ts">
import type { ServerRow } from '~/types/api'
import type { PermissionEntry } from '~/types/permissions'

const props = defineProps<{
  title: string
  subtitle: string
  entries: PermissionEntry[]
  servers: ServerRow[]
  busy: string | null
}>()

const emit = defineEmits<{
  add: [entries: PermissionEntry[]]
  remove: [entries: PermissionEntry[]]
}>()

const { t } = useT()
const node = ref('')
const contextDropdownOpen = ref(false)
const global = ref(true)
const picked = ref<string[]>([])

const contextSummary = computed(() => {
  if (global.value) return t('admin-perm-all-builds') || 'Все сборки'
  if (picked.value.length === 0) return 'Выберите...'
  if (picked.value.length === 1) return props.servers.find(s => s.id === picked.value[0])?.name || picked.value[0]
  return `Выбрано: ${picked.value.length}`
})

const catalogFor = computed(() => (global.value ? '' : picked.value[0] || ''))
const { suggestions, pending, error } = usePermissionNodes(catalogFor)

const PRESETS = [
  { key: 'admin-tokens-preset-superadmin', hint: '*', nodes: ['*'] },
  { key: 'admin-tokens-preset-fulladmin', hint: 'noro.admin.*', nodes: ['noro.admin.*'] },
  { key: 'admin-tokens-preset-senior-mod', hint: 'noro.mod.*', nodes: ['noro.mod.*', 'noro.admin.users.view', 'noro.admin.rules.view', 'noro.admin.audit'] },
  { key: 'admin-tokens-preset-junior-mod', hint: '', nodes: ['noro.mod.punish.warn', 'noro.mod.punish.mute', 'noro.mod.cases.view', 'noro.mod.cases.claim', 'noro.mod.cases.resolve', 'noro.admin.users.view', 'noro.admin.rules.view'] },
] as const

const targets = computed<(string | null)[]>(() =>
  global.value ? [null] : picked.value.slice()
)
const labels = computed(() => new Map(suggestions.value.map(s => [s.node, s.label])))
const fresh = computed(() => targets.value.filter(server =>
  !props.entries.some(e => e.permission === node.value.trim() && e.server_id === server)
))
const rows = computed(() => {
  const byPerm = new Map<string, (string | null)[]>()
  for (const e of props.entries) {
    byPerm.set(e.permission, [...(byPerm.get(e.permission) || []), e.server_id])
  }
  return [...byPerm.entries()]
    .map(([permission, contexts]) => ({ permission, contexts }))
    .sort((a, b) => a.permission.localeCompare(b.permission))
})

function add() {
  const p = node.value.trim()
  if (!p || !fresh.value.length) return
  emit('add', fresh.value.map(server_id => ({ permission: p, server_id })))
  node.value = ''
}
function toggle(permission: string, serverId: string | null) {
  const entry = { permission, server_id: serverId }
  const granted = props.entries.some(e => e.permission === permission && e.server_id === serverId)
  if (granted) emit('remove', [entry])
  else emit('add', [entry])
}
function applyPreset(nodesList: readonly string[]) {
  const sids = global.value ? [null] : picked.value.slice()
  const toAdd = nodesList.flatMap(p => 
    sids.filter(sid => !props.entries.some(e => e.permission === p && e.server_id === sid))
        .map(sid => ({ permission: p, server_id: sid }))
  )
  if (toAdd.length) emit('add', toAdd)
}
function removeAll(permission: string) {
  emit('remove', props.entries.filter(e => e.permission === permission))
}
</script>

<template>
  <section class="noro-panel p-5">
    <h2 class="text-xl font-black text-[var(--noro-text)]">{{ title }}</h2>
    <p class="mt-1 text-sm text-[var(--noro-muted)]">{{ subtitle }}</p>

    <!-- Presets -->
    <div class="mt-4 grid gap-1.5">
      <span class="noro-label">{{ t('admin-tokens-preset-title') }}</span>
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="p in PRESETS" :key="p.key"
          type="button"
          class="group flex items-center gap-1.5 rounded-md border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] px-2.5 py-1.5 text-xs font-semibold text-[var(--noro-muted)] transition hover:border-[var(--noro-cream)] hover:text-[var(--noro-text)] cursor-pointer"
          @click="applyPreset(p.nodes)"
        >
          <span>{{ t(p.key) }}</span>
          <span v-if="p.hint" class="font-mono text-[10px] text-[var(--noro-muted)] opacity-60 group-hover:opacity-100">{{ p.hint }}</span>
        </button>
      </div>
    </div>

    <!-- Input -->
    <div class="mt-5">
      <span class="noro-label">{{ t('admin-perm-permission') }}</span>
      <div class="mt-1 flex items-start gap-2">
        <div class="relative w-44 shrink-0">
          <button
            type="button"
            class="noro-input w-full bg-[var(--noro-input)] text-xs font-semibold text-[var(--noro-text)] cursor-pointer flex items-center justify-between"
            @click="contextDropdownOpen = !contextDropdownOpen"
          >
            <span class="truncate block">{{ contextSummary }}</span>
            <UIcon name="i-lucide-chevron-down" class="shrink-0 ml-2" />
          </button>

          <Teleport to="body">
            <div
              v-if="contextDropdownOpen"
              class="fixed inset-0 z-40"
              @click="contextDropdownOpen = false"
            ></div>
          </Teleport>
          
          <div
            v-if="contextDropdownOpen"
            class="absolute top-full left-0 mt-1.5 w-56 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-panel-2)] p-1.5 shadow-xl z-50 flex flex-col gap-1 max-h-60 overflow-y-auto noro-scroll"
          >
            <label class="flex items-center gap-2.5 rounded px-2 py-1.5 cursor-pointer hover:bg-[var(--noro-input)] transition">
              <input
                type="checkbox"
                class="mt-0.5 size-3.5 rounded border-[var(--noro-border)] bg-[var(--noro-input)] text-[var(--noro-cream)]"
                v-model="global"
              >
              <span class="text-xs font-semibold text-[var(--noro-text)]">{{ t('admin-perm-all-builds') }}</span>
            </label>
            <div class="h-px bg-[var(--noro-border)] mx-1"></div>
            <label
              v-for="server in servers"
              :key="server.id"
              class="flex items-center gap-2.5 rounded px-2 py-1.5 cursor-pointer hover:bg-[var(--noro-input)] transition"
              :class="global ? 'opacity-50 pointer-events-none' : ''"
            >
              <input
                type="checkbox"
                class="mt-0.5 size-3.5 rounded border-[var(--noro-border)] bg-[var(--noro-input)] text-[var(--noro-cream)]"
                :value="server.id"
                v-model="picked"
                :disabled="global"
              >
              <span class="text-xs font-semibold text-[var(--noro-text)] truncate">{{ server.name }}</span>
            </label>
          </div>
        </div>
        <div class="min-w-0 flex-1">
          <AdminPermissionInput v-model="node" :suggestions="suggestions" :loading="pending" @submit="add" />
        </div>
        <AtomButton
          variant="primary" icon="i-lucide-plus" class="shrink-0"
          :loading="busy === 'perm'" :disabled="!node.trim() || !fresh.length || busy === 'perm'"
          @click="add"
        >
          {{ t('admin-perm-add') }}
        </AtomButton>
      </div>
    </div>

    <p v-if="error" class="mt-3 text-xs text-[var(--noro-amber)]">
      {{ t('admin-perm-suggestions-unavailable', { error: String(error) }) }}
    </p>
    <p v-else-if="node.trim() && !fresh.length" class="mt-3 text-xs text-[var(--noro-amber)]">
      {{ t('admin-perm-already-granted') }}
    </p>

    <!-- Granted list -->
    <div class="mt-5 grid gap-2">
      <AdminPermissionRow
        v-for="row in rows" :key="row.permission"
        :permission="row.permission" :contexts="row.contexts" :servers="servers"
        :label="labels.get(row.permission)" :busy="busy === `perm-${row.permission}`"
        @toggle="toggle(row.permission, $event)" @remove="removeAll(row.permission)"
      />
      <p v-if="!entries.length" class="rounded-lg bg-[var(--noro-input)] px-3 py-4 text-center text-sm text-[var(--noro-muted)]">
        {{ t('admin-perm-no-permissions') }}
      </p>
    </div>
  </section>
</template>
