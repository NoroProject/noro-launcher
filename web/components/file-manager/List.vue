<script setup lang="ts">
export interface FmItem {
  type: 'folder' | 'file'
  name: string
  path: string
  id?: string
  sha1?: string
  size?: number
  kind?: string
  count?: number
}

const props = defineProps<{
  items: FmItem[]
  rule: (path: string) => RuleState
  selected: Set<string>
  sortKey: string
  sortAsc: boolean
  renamingId: string | null
  renameValue: string
  modeHint?: Record<SyncMode, string>
}>()

const { t } = useT()

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

const cols = computed(() => [
  { key: 'name', label: t('admin-fm-col-name'), cls: '' },
  { key: 'sync', label: t('admin-fm-col-sync'), cls: 'w-16 text-center' },
  { key: 'size', label: t('admin-fm-col-size'), cls: 'w-24 text-right' },
  { key: 'kind', label: t('admin-fm-col-kind'), cls: 'w-28' },
])

const MODE_CLASS: Record<SyncMode, string> = {
  sync: 'text-[var(--noro-cream)]',
  ignored: 'text-[var(--noro-blue)]',
  user: 'text-[var(--noro-magenta)]',
}

function ruleTitle(path: string) {
  const r = props.rule(path)
  const hint = props.modeHint ? props.modeHint[r.mode] : r.mode
  return r.from ? `${hint} (inherited from ${r.from})` : hint
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
          <td class="text-center">
            <button
              class="font-mono text-xs font-bold hover:opacity-100"
              :class="[MODE_CLASS[rule(it.path).mode], rule(it.path).from ? 'opacity-40' : '']"
              :title="ruleTitle(it.path)"
              @click.stop="emit('toggle-rule', it.path)"
            >
              {{ MODE_LABEL[rule(it.path).mode] }}
            </button>
          </td>
          <td class="text-right text-[var(--noro-muted)]">
            {{ it.type === 'file' ? fmtSize(it.size) : `${it.count} ${t('admin-fm-items')}` }}
          </td>
          <td class="text-[var(--noro-muted)]">
            {{ it.type === 'folder' ? t('admin-fm-folder') : (it.kind || 'File') }}
          </td>
        </tr>
        <tr v-if="!items.length">
          <td colspan="4" class="text-center text-[var(--noro-muted)] py-8">
            {{ t('admin-fm-empty') }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
