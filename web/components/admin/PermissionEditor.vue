<script setup lang="ts">
import type { ServerRow } from '~/types/api'
import type { PermissionEntry } from '~/types/permissions'
import { permissionKey } from '~/types/permissions'

const props = defineProps<{
  title: string
  subtitle: string
  entries: PermissionEntry[]
  servers: ServerRow[]
  busy: string | null
}>()

const emit = defineEmits<{
  add: [entries: PermissionEntry[]]
  remove: [entry: PermissionEntry]
  move: [entry: PermissionEntry, serverId: string | null]
}>()

const node = ref('')
/** Пустая строка — «на всех сборках». Иначе список выбранных сборок. */
const picked = ref<string[]>([])
const global = ref(true)

/** Каталог узлов зависит от сборки; берём первую выбранную как образец. */
const catalogFor = computed(() => (global.value ? '' : picked.value[0] || ''))
const { suggestions, pending, error } = usePermissionNodes(catalogFor)

/** Контексты, в которые уйдёт право. Глобальный перекрывает все остальные. */
const targets = computed<(string | null)[]>(() =>
  global.value ? [null] : picked.value.slice()
)

/** Пояснения к уже выданным правам берём из того же каталога. */
const labels = computed(() => new Map(suggestions.value.map(item => [item.node, item.label])))

/** Дубликатом считаем только те контексты, где право уже есть. */
const fresh = computed(() => targets.value.filter(server =>
  !props.entries.some(e => e.permission === node.value.trim() && e.server_id === server)
))

/** Группируем по контексту: глобальные сверху, дальше сборки по алфавиту. */
const groups = computed(() => {
  const byContext = new Map<string, PermissionEntry[]>()
  for (const entry of props.entries) {
    const key = entry.server_id || ''
    byContext.set(key, [...(byContext.get(key) || []), entry])
  }
  return [...byContext.entries()]
    .map(([key, items]) => ({
      key,
      name: contextName(key),
      items: [...items].sort((a, b) => a.permission.localeCompare(b.permission))
    }))
    .sort((a, b) => Number(Boolean(a.key)) - Number(Boolean(b.key)) || a.name.localeCompare(b.name))
})

function contextName(serverId: string) {
  if (!serverId) return 'Global'
  return props.servers.find(server => server.id === serverId)?.name || 'Unknown build'
}

function add() {
  const permission = node.value.trim()
  if (!permission || !fresh.value.length) return
  emit('add', fresh.value.map(server_id => ({ permission, server_id })))
  node.value = ''
}
</script>

<template>
  <section class="noro-panel p-5">
    <h2 class="text-xl font-black text-[var(--noro-text)]">{{ title }}</h2>
    <p class="mt-1 text-sm text-[var(--noro-muted)]">{{ subtitle }}</p>

    <!-- Две строки, а не одна: чипы контекста растут по числу сборок, и держать
         их в колонке рядом с полем — значит ломать раскладку на третьей сборке. -->
    <div class="mt-4 grid gap-3">
      <AdminPermissionContextPicker v-model:global="global" v-model:picked="picked" :servers="servers" />
      <div>
        <span class="noro-label">Permission</span>
        <div class="mt-1 flex items-start gap-2">
          <div class="min-w-0 flex-1">
            <AdminPermissionInput
              v-model="node"
              :suggestions="suggestions"
              :loading="pending"
              @submit="add"
            />
          </div>
          <button
            type="button"
            class="noro-btn noro-btn-primary shrink-0"
            :disabled="!node.trim() || !fresh.length || busy === 'perm'"
            @click="add"
          >
            <UIcon
              :name="busy === 'perm' ? 'i-lucide-loader-circle' : 'i-lucide-plus'"
              class="size-4"
              :class="busy === 'perm' ? 'animate-spin' : ''"
            />
            Add
          </button>
        </div>
      </div>
    </div>

    <p v-if="error" class="mt-3 text-xs text-[var(--noro-amber)]">
      Suggestions unavailable: {{ error }}. Permissions can still be typed by hand.
    </p>
    <p v-else-if="node.trim() && !fresh.length" class="mt-3 text-xs text-[var(--noro-amber)]">
      Already granted everywhere you picked.
    </p>
    <p v-else-if="!global && !picked.length" class="mt-3 text-xs text-[var(--noro-muted)]">
      Pick at least one build, or grant it on all of them.
    </p>

    <div class="mt-5 grid gap-4">
      <div v-for="group in groups" :key="group.key">
        <div class="mb-2 flex items-center gap-2">
          <UIcon :name="group.key ? 'i-lucide-box' : 'i-lucide-globe'" class="size-4 text-[var(--noro-muted)]" />
          <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--noro-muted)]">{{ group.name }}</h3>
          <span class="rounded bg-[var(--noro-input)] px-2 py-0.5 text-xs text-[var(--noro-muted)]">
            {{ group.items.length }}
          </span>
        </div>
        <div class="grid gap-2">
          <AdminPermissionRow
            v-for="entry in group.items"
            :key="permissionKey(entry)"
            :entry="entry"
            :servers="servers"
            :label="labels.get(entry.permission)"
            :busy="busy === `perm-${entry.permission}`"
            @remove="emit('remove', entry)"
            @move="emit('move', entry, $event)"
          />
        </div>
      </div>

      <p v-if="!entries.length" class="rounded-lg bg-[var(--noro-input)] px-3 py-4 text-center text-sm text-[var(--noro-muted)]">
        No permissions granted yet.
      </p>
    </div>
  </section>
</template>
