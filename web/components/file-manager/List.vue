<script setup lang="ts">
export interface FmItem {
  type: 'folder' | 'file'
  name: string
  /** Полный путь от корня сборки; у папок — с косой чертой на конце. */
  path: string
  id?: string
  /** Адрес в контент-адресуемом хранилище: по нему файл и скачивается. */
  sha1?: string
  size?: number
  kind?: string
  count?: number
}

const props = defineProps<{
  items: FmItem[]
  /** Режим синхронизации пути с учётом наследования от папок. */
  rule: (path: string) => RuleState
  selected: Set<string>
  sortKey: string
  sortAsc: boolean
  renamingId: string | null
  renameValue: string
}>()

const emit = defineEmits<{
  select: [id: string, ev: MouseEvent]
  open: [item: FmItem]
  context: [item: FmItem, ev: MouseEvent]
  sort: [key: string]
  'toggle-rule': [path: string]
  'bg-context': [ev: MouseEvent]
  'update:renameValue': [v: string]
  'rename-commit': []
  'rename-cancel': []
}>()

function itemKey(it: FmItem) {
  return it.type === 'folder' ? `d:${it.name}` : `f:${it.id}`
}

function fmtSize(b?: number) {
  if (b == null) return ''
  if (b < 1024) return `${b} B`
  if (b < 1048576) return `${(b / 1024).toFixed(1)} KB`
  return `${(b / 1048576).toFixed(1)} MB`
}

const cols = [
  { key: 'name', label: 'Name', cls: '' },
  { key: 'sync', label: 'Sync', cls: 'w-16 text-center' },
  { key: 'size', label: 'Size', cls: 'w-24 text-right' },
  { key: 'kind', label: 'Kind', cls: 'w-28' },
]

const MODE_CLASS: Record<SyncMode, string> = {
  sync: 'text-[var(--noro-cream)]',
  ignored: 'text-[var(--noro-blue)]',
  user: 'text-[var(--noro-magenta)]',
}

/** Подпись как у прав доступа: буква режима плюс откуда он взялся. */
function ruleTitle(path: string) {
  const r = props.rule(path)
  return r.from ? `${MODE_HINT[r.mode]} (inherited from ${r.from})` : MODE_HINT[r.mode]
}

function sortIcon(key: string) {
  if (props.sortKey !== key) return ''
  return props.sortAsc ? '↑' : '↓'
}

function iconForKind(it: FmItem) {
  if (it.type === 'folder') return 'i-lucide-folder'
  const ext = it.name.split('.').pop()?.toLowerCase() || ''
  const map: Record<string, string> = {
    jar: 'i-lucide-package',
    json: 'i-lucide-braces',
    toml: 'i-lucide-settings',
    cfg: 'i-lucide-settings',
    png: 'i-lucide-image',
    jpg: 'i-lucide-image',
    txt: 'i-lucide-file-text',
    log: 'i-lucide-file-text',
    yml: 'i-lucide-file-code',
    yaml: 'i-lucide-file-code',
    properties: 'i-lucide-file-code',
  }
  return map[ext] || 'i-lucide-file'
}

function iconColor(it: FmItem) {
  if (it.type === 'folder') return 'text-[var(--noro-amber)]'
  return 'text-[var(--noro-blue)]'
}
</script>

<template>
  <div @contextmenu.prevent="emit('bg-context', $event)">
    <table class="fm-list">
      <thead>
        <tr>
          <th
            v-for="c in cols"
            :key="c.key"
            :class="c.cls"
            @click="emit('sort', c.key)"
          >
            {{ c.label }} {{ sortIcon(c.key) }}
          </th>
        </tr>
      </thead>
      <tbody>
        <!--
          .stop у contextmenu обязателен: без него событие всплывает к
          обработчику фона, который сбрасывает выделение, и меню открывается
          без действий над файлом. В Grid.vue это уже учтено.
        -->
        <tr
          v-for="it in items"
          :key="itemKey(it)"
          :class="{ 'fm-selected': selected.has(itemKey(it)) }"
          @click="emit('select', itemKey(it), $event)"
          @dblclick="emit('open', it)"
          @contextmenu.prevent.stop="emit('context', it, $event)"
        >
          <td>
            <span class="inline-flex items-center gap-2 min-w-0">
              <UIcon :name="iconForKind(it)" :class="['size-4 shrink-0', iconColor(it)]" />
              <template v-if="renamingId === itemKey(it)">
                <input
                  class="fm-rename-input"
                  :value="renameValue"
                  @input="emit('update:renameValue', ($event.target as HTMLInputElement).value)"
                  @keydown.enter="emit('rename-commit')"
                  @keydown.escape="emit('rename-cancel')"
                  @blur="emit('rename-commit')"
                  autofocus
                />
              </template>
              <template v-else>
                <span class="truncate">{{ it.name }}</span>
                <span v-if="it.type === 'folder'" class="text-xs text-[var(--noro-muted)]">({{ it.count }})</span>
              </template>
            </span>
          </td>
          <td class="text-right text-[var(--noro-muted)]">
            {{ it.type === 'file' ? fmtSize(it.size) : `${it.count} items` }}
          </td>
          <td class="text-[var(--noro-muted)]">
            {{ it.type === 'folder' ? 'Folder' : (it.kind || 'File') }}
          </td>
        </tr>
        <tr v-if="!items.length">
          <td colspan="3" class="text-center text-[var(--noro-muted)] py-8">
            Empty folder
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
