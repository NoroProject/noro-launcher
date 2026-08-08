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

/**
 * Одна строка на право, а не на выдачу: право живёт сразу в нескольких
 * контекстах, и раскладка по контекстам разносила его копии по разным местам
 * списка — понять, где оно вообще действует, было нельзя.
 */
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

function add() {
  const permission = node.value.trim()
  if (!permission || !fresh.value.length) return
  emit('add', fresh.value.map(server_id => ({ permission, server_id })))
  node.value = ''
}

/** Клик по чипу: где право уже есть — снимаем, где нет — выдаём. */
function toggle(permission: string, serverId: string | null) {
  const entry = { permission, server_id: serverId }
  const granted = props.entries.some(
    e => e.permission === permission && e.server_id === serverId
  )
  // Две ветки, а не имя события выражением: с вычисленным именем перегрузка
  // `emit` не выводится и типы событий перестают проверяться вовсе.
  if (granted) {
    emit('remove', [entry])
  } else {
    emit('add', [entry])
  }
}

/** Крестик снимает право целиком — во всех контекстах сразу. */
function removeAll(permission: string) {
  emit('remove', props.entries.filter(e => e.permission === permission))
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
          <AtomButton
            variant="primary"
            icon="i-lucide-plus"
            :loading="busy === 'perm'"
            :disabled="!node.trim() || !fresh.length || busy === 'perm'"
            @click="add"
            class="shrink-0"
          >
            Add
          </AtomButton>
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
        No permissions granted yet.
      </p>
    </div>
  </section>
</template>
