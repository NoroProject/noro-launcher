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
const picked = ref<string[]>([])
const global = ref(true)
const showPicker = ref(false)

const catalogFor = computed(() => (global.value ? '' : picked.value[0] || ''))
const { suggestions, pending, error } = usePermissionNodes(catalogFor)

const PRESETS = computed(() => [
  { id: 'superadmin', label: '👑 ' + t('admin-tokens-preset-superadmin'), nodes: ['*'] },
  { id: 'fulladmin', label: '🛡️ ' + t('admin-tokens-preset-fulladmin'), nodes: ['noro.admin.*'] },
  { id: 'seniormod', label: '⚔️ ' + t('admin-tokens-preset-senior-mod'), nodes: ['noro.mod.*', 'noro.admin.users.view', 'noro.admin.rules.view', 'noro.admin.audit'] },
  { id: 'juniormod', label: '🤝 ' + t('admin-tokens-preset-junior-mod'), nodes: ['noro.mod.punish.warn', 'noro.mod.punish.mute', 'noro.mod.cases.view', 'noro.mod.cases.claim', 'noro.mod.cases.resolve', 'noro.admin.users.view', 'noro.admin.rules.view'] },
])

const groups = computed(() => {
  const map = new Map<string, { name: string, title: string }[]>()
  for (const s of suggestions.value) {
    const grp = s.group || 'System'
    const list = map.get(grp) || []
    list.push({ name: s.node, title: s.label || s.node })
    map.set(grp, list)
  }
  return [...map.entries()].map(([title, items]) => ({ title, items }))
})

const targets = computed<(string | null)[]>(() =>
  global.value ? [null] : picked.value.slice()
)

const labels = computed(() => new Map(suggestions.value.map(item => [item.node, item.label])))

const fresh = computed(() => targets.value.filter(server =>
  !props.entries.some(e => e.permission === node.value.trim() && e.server_id === server)
))

const rows = computed(() => {
  const byPermission = new Map<string, (string | null)[]>()
  for (const entry of props.entries) {
    byPermission.set(entry.permission, [
      ...(byPermission.get(entry.permission) || []),
      entry.server_id
    ])
  }
  return [...byPermission.entries()]
    .map(([permission, contexts]) => ({ permission, contexts }))
    .sort((a, b) => a.permission.localeCompare(b.permission))
})

function isGranted(nodeName: string) {
  const currentServer = global.value ? null : (picked.value[0] || null)
  return props.entries.some(e => e.permission === nodeName && e.server_id === currentServer)
}

function add() {
  const permission = node.value.trim()
  if (!permission || !fresh.value.length) return
  emit('add', fresh.value.map(server_id => ({ permission, server_id })))
  node.value = ''
}

function toggle(permission: string, serverId: string | null) {
  const entry = { permission, server_id: serverId }
  const granted = props.entries.some(
    e => e.permission === permission && e.server_id === serverId
  )
  if (granted) {
    emit('remove', [entry])
  } else {
    emit('add', [entry])
  }
}

function applyPreset(nodesList: string[]) {
  const serverId = global.value ? null : (picked.value[0] || null)
  const toAdd = nodesList
    .filter(p => !props.entries.some(e => e.permission === p && e.server_id === serverId))
    .map(permission => ({ permission, server_id: serverId }))
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

    <!-- Presets & Category Picker Toggle -->
    <div class="mt-4 flex flex-wrap items-center justify-between gap-2 border-b border-[var(--noro-border)] pb-3">
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="p in PRESETS"
          :key="p.id"
          type="button"
          class="rounded-md px-2 py-1 text-xs font-bold transition border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] text-[var(--noro-muted)] hover:text-[var(--noro-text)] hover:border-[var(--noro-cream)] cursor-pointer"
          @click="applyPreset(p.nodes)"
        >
          {{ p.label }}
        </button>
      </div>
      <button
        type="button"
        class="text-xs text-[var(--noro-cream)] hover:underline flex items-center gap-1 font-bold cursor-pointer"
        @click="showPicker = !showPicker"
      >
        <UIcon :name="showPicker ? 'i-lucide-chevron-up' : 'i-lucide-grid'" class="size-4" />
        {{ showPicker ? 'Скрыть категории' : 'Показать все узлы' }}
      </button>
    </div>

    <!-- Grouped Categories Grid -->
    <div v-if="showPicker" class="mt-3 max-h-72 overflow-y-auto noro-scroll rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] p-3 grid gap-4">
      <div v-for="grp in groups" :key="grp.title" class="grid gap-2">
        <div class="flex items-center justify-between border-b border-[var(--noro-border)] pb-1">
          <span class="text-xs font-black uppercase text-[var(--noro-cream)]">{{ grp.title }}</span>
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-1.5">
          <label
            v-for="item in grp.items"
            :key="item.name"
            class="flex items-start gap-2 rounded px-2 py-1 transition cursor-pointer hover:bg-[var(--noro-panel)]"
          >
            <input
              type="checkbox"
              class="mt-0.5 size-3.5 rounded border-[var(--noro-border)] bg-[var(--noro-input)] text-[var(--noro-cream)]"
              :checked="isGranted(item.name)"
              @change="toggle(item.name, global ? null : (picked[0] || null))"
            >
            <div class="min-w-0 flex-1">
              <div class="text-xs font-semibold text-[var(--noro-text)] leading-snug">{{ item.title }}</div>
              <div class="text-[10px] font-mono text-[var(--noro-muted)] truncate">{{ item.name }}</div>
            </div>
          </label>
        </div>
      </div>
    </div>

    <div class="mt-4 grid gap-3">
      <AdminPermissionContextPicker v-model:global="global" v-model:picked="picked" :servers="servers" />
      <div>
        <span class="noro-label">{{ t('admin-perm-permission') }}</span>
        <div class="mt-1 flex items-start gap-2">
          <div class="min-w-0 flex-1">
            <AdminPermissionInput
              v-model="node"
              :suggestions="suggestions"
              :loading="pending"
              @submit="add"
            />
          </div>
          <AtomButton
            variant="primary"
            icon="i-lucide-plus"
            :loading="busy === 'perm'"
            :disabled="!node.trim() || !fresh.length || busy === 'perm'"
            @click="add"
            class="shrink-0"
          >
            {{ t('admin-perm-add') }}
          </AtomButton>
        </div>
      </div>
    </div>

    <p v-if="error" class="mt-3 text-xs text-[var(--noro-amber)]">
      {{ t('admin-perm-suggestions-unavailable', { error: String(error) }) }}
    </p>
    <p v-else-if="node.trim() && !fresh.length" class="mt-3 text-xs text-[var(--noro-amber)]">
      {{ t('admin-perm-already-granted') }}
    </p>
    <p v-else-if="!global && !picked.length" class="mt-3 text-xs text-[var(--noro-muted)]">
      {{ t('admin-perm-pick-build') }}
    </p>

    <div class="mt-5 grid gap-2">
      <AdminPermissionRow
        v-for="row in rows"
        :key="row.permission"
        :permission="row.permission"
        :contexts="row.contexts"
        :servers="servers"
        :label="labels.get(row.permission)"
        :busy="busy === `perm-${row.permission}`"
        @toggle="toggle(row.permission, $event)"
        @remove="removeAll(row.permission)"
      />

      <p v-if="!entries.length" class="rounded-lg bg-[var(--noro-input)] px-3 py-4 text-center text-sm text-[var(--noro-muted)]">
        {{ t('admin-perm-no-permissions') }}
      </p>
    </div>
  </section>
</template>
