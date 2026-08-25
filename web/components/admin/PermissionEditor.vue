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
const contextAnchor = ref<HTMLElement | null>(null)
const { style: contextPopupStyle } = useAnchoredPopup(contextAnchor, contextDropdownOpen)

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
function toggleGlobal() {
  global.value = !global.value
}
function togglePicked(serverId: string) {
  if (global.value) global.value = false
  const idx = picked.value.indexOf(serverId)
  if (idx > -1) {
    picked.value.splice(idx, 1)
  } else {
    picked.value.push(serverId)
  }
}
function toggle(permission: string, serverId: string | null) {
  const entry = { permission, server_id: serverId }
  const granted = props.entries.some(e => e.permission === permission && e.server_id === serverId)
  if (granted) emit('remove', [entry])
  else emit('add', [entry])
}
function removeAll(permission: string) {
  emit('remove', props.entries.filter(e => e.permission === permission))
}
</script>

<template>
  <section class="noro-panel p-5">
    <h2 class="text-xl font-black text-[var(--noro-text)]">{{ title }}</h2>
    <p class="mt-1 text-sm text-[var(--noro-muted)]">{{ subtitle }}</p>

    <!-- Input -->
    <div class="mt-5">
      <span class="noro-label">{{ t('admin-perm-permission') }}</span>
      <div class="mt-1 flex items-start gap-2">
        <div class="relative w-44 shrink-0">
          <button
            ref="contextAnchor"
            type="button"
            class="noro-input w-full !flex items-center justify-between bg-[var(--noro-input)] text-xs font-semibold text-[var(--noro-text)] cursor-pointer"
            @click="contextDropdownOpen = !contextDropdownOpen"
          >
            <span class="truncate block">{{ contextSummary }}</span>
            <UIcon name="i-lucide-chevron-down" class="shrink-0 ml-2 text-[var(--noro-muted)]" />
          </button>

          <Teleport to="body">
            <!-- Invisible backdrop to catch outside clicks without z-index issues -->
            <div
              v-if="contextDropdownOpen"
              class="fixed inset-0 z-40"
              @mousedown="contextDropdownOpen = false"
            ></div>
            
            <div
              v-if="contextDropdownOpen"
              class="absolute z-50 mt-1.5 w-56 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-panel-2)] p-1 shadow-xl flex flex-col max-h-60 overflow-y-auto noro-scroll"
              :style="contextPopupStyle"
              @mousedown.stop
            >
              <button
                type="button"
                class="flex items-center justify-between rounded px-2.5 py-2 text-xs font-semibold transition hover:bg-[var(--noro-input)]"
                :class="global ? 'text-[var(--noro-blue)]' : 'text-[var(--noro-text)]'"
                @click="toggleGlobal"
              >
                <span>{{ t('admin-perm-all-builds') }}</span>
                <UIcon v-if="global" name="i-lucide-check" class="size-4 shrink-0" />
              </button>
              
              <div class="h-px bg-[var(--noro-border)] mx-1 my-1"></div>
              
              <button
                v-for="server in servers"
                :key="server.id"
                type="button"
                class="flex items-center justify-between rounded px-2.5 py-2 text-xs font-semibold transition hover:bg-[var(--noro-input)]"
                :class="[
                  global ? 'opacity-50 cursor-not-allowed' : '',
                  !global && picked.includes(server.id) ? 'text-[var(--noro-blue)]' : 'text-[var(--noro-text)]'
                ]"
                :disabled="global"
                @click="togglePicked(server.id)"
              >
                <span class="truncate">{{ server.name }}</span>
                <UIcon v-if="!global && picked.includes(server.id)" name="i-lucide-check" class="size-4 shrink-0" />
              </button>
            </div>
          </Teleport>
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
