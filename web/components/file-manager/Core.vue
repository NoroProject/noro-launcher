<script setup lang="ts">
import type { BuildFileRow } from '~/types/api'
import type { FmItem } from './List.vue'

const props = defineProps<{ buildId: string }>()
const emit = defineEmits<{ changed: [] }>()
const auth = useAuth()
const notify = useNotify()

// ─── State ───
const files = ref<BuildFileRow[]>([])
const loading = ref(false)
const currentPath = ref('')
const search = ref('')
const viewMode = ref<'list' | 'grid'>('list')
const selected = ref(new Set<string>())
const sortKey = ref('name')
const sortAsc = ref(true)
const dragging = ref(false)
const renamingId = ref<string | null>(null)
const renameValue = ref('')

// Context menu
const ctxVisible = ref(false)
const ctxX = ref(0)
const ctxY = ref(0)
const ctxItem = ref<FmItem | null>(null)
const ctxIsText = computed(() => {
  if (!ctxItem.value || ctxItem.value.type === 'folder') return false
  const ext = ctxItem.value.name.split('.').pop()?.toLowerCase() || ''
  return ['json', 'toml', 'cfg', 'txt', 'yml', 'yaml', 'properties', 'xml'].includes(ext)
})

// Editor state
const editorVisible = ref(false)
const editorFileId = ref('')
const editorFileName = ref('')
const editorFilePath = ref('')

// ─── Правила синхронизации ───
const rules = useSyncRules(props.buildId)

/** Клик по метке меняет режим и сразу сохраняет — как chmod, без «применить». */
/** Выбор режима из контекстного меню — без перебора по кругу. */
async function setSyncFromMenu(mode: SyncMode) {
  if (!ctxItem.value) return
  const path = ctxItem.value.path
  ctxVisible.value = false
  rules.setMode(path, mode)
  try {
    await rules.save()
  } catch (e) {
    notify.fail(e)
    await rules.load()
  }
}

async function toggleRule(path: string) {
  rules.cycle(path)
  try {
    await rules.save()
  } catch (e) {
    notify.fail(e)
    await rules.load()
  }
}

// ─── API ───
async function loadFiles() {
  loading.value = true
  try {
    files.value = await auth.request(`/api/admin/builds/${props.buildId}/files`)
  } finally { loading.value = false }
}

onMounted(() => Promise.all([loadFiles(), rules.load()]))

// ─── Breadcrumb ───
const breadcrumb = computed(() => {
  const res = [{ name: 'Root', path: '' }]
  if (!currentPath.value) return res
  let acc = ''
  for (const p of currentPath.value.replace(/\/$/, '').split('/')) {
    acc += p + '/'
    res.push({ name: p, path: acc })
  }
  return res
})

function goBack() {
  const parts = currentPath.value.replace(/\/$/, '').split('/').filter(Boolean)
  parts.pop()
  currentPath.value = parts.length ? parts.join('/') + '/' : ''
}

// ─── Items ───
const currentItems = computed<FmItem[]>(() => {
  const prefix = currentPath.value
  const folderMap = new Map<string, { count: number; size: number }>()
  const fileItems: FmItem[] = []

  for (const f of files.value) {
    if (!f.path.startsWith(prefix)) continue
    const rel = f.path.slice(prefix.length)
    if (!rel) continue
    const slash = rel.indexOf('/')
    if (slash >= 0) {
      const dir = rel.slice(0, slash)
      const entry = folderMap.get(dir) || { count: 0, size: 0 }
      entry.count++
      entry.size += f.size || 0
      folderMap.set(dir, entry)
    } else {
      const ext = rel.split('.').pop()?.toUpperCase() || ''
      fileItems.push({
        type: 'file', name: rel, path: f.path, id: f.id, sha1: f.sha1,
        size: f.size, kind: ext || 'File',
      })
    }
  }

  // Путь папки — с косой чертой: так правило и наследуется вниз.
  const folderItems: FmItem[] = Array.from(folderMap, ([name, info]) => ({
    type: 'folder', name, path: `${prefix}${name}/`, count: info.count,
  }))

  let all = [...folderItems, ...fileItems]
  const q = search.value.toLowerCase()
  if (q) all = all.filter(it => it.name.toLowerCase().includes(q))
  all.sort((a, b) => {
    if (a.type !== b.type) return a.type === 'folder' ? -1 : 1
    const key = sortKey.value as keyof FmItem
    const av = a[key] ?? '', bv = b[key] ?? ''
    const cmp = typeof av === 'number' && typeof bv === 'number'
      ? av - bv
      : String(av).localeCompare(String(bv))
    return sortAsc.value ? cmp : -cmp
  })
  return all
})

// ─── Selection ───
function handleSelect(id: string, ev: MouseEvent) {
  if (ev.metaKey || ev.ctrlKey) {
    const s = new Set(selected.value)
    s.has(id) ? s.delete(id) : s.add(id)
    selected.value = s
  } else if (ev.shiftKey && selected.value.size > 0) {
    const keys = currentItems.value.map(it => it.type === 'folder' ? `d:${it.name}` : `f:${it.id}`)
    const last = Array.from(selected.value).pop()!
    const a = keys.indexOf(last), b = keys.indexOf(id)
    const [lo, hi] = a < b ? [a, b] : [b, a]
    selected.value = new Set(keys.slice(lo, hi + 1))
  } else {
    selected.value = new Set([id])
  }
}

function selectAll() {
  selected.value = new Set(
    currentItems.value.map(it => it.type === 'folder' ? `d:${it.name}` : `f:${it.id}`)
  )
}

// ─── Open ───
function openItem(it: FmItem) {
  if (it.type === 'folder') {
    currentPath.value += it.name + '/'
    selected.value = new Set()
  }
}

// ─── Sort ───
function toggleSort(key: string) {
  if (sortKey.value === key) sortAsc.value = !sortAsc.value
  else { sortKey.value = key; sortAsc.value = true }
}

// ─── Context menu ───
function showContext(it: FmItem, ev: MouseEvent) {
  ctxItem.value = it
  const key = it.type === 'folder' ? `d:${it.name}` : `f:${it.id}`
  if (!selected.value.has(key)) selected.value = new Set([key])
  ctxX.value = ev.clientX
  ctxY.value = ev.clientY
  ctxVisible.value = true
}

function showBgContext(ev: MouseEvent) {
  ctxItem.value = null
  selected.value = new Set()
  ctxX.value = ev.clientX
  ctxY.value = ev.clientY
  ctxVisible.value = true
}

function closeCtx() { ctxVisible.value = false }

// ─── Actions ───
async function deleteSelected() {
  if (!confirm(`Delete ${selected.value.size} item(s)?`)) return
  loading.value = true
  try {
    for (const key of selected.value) {
      if (key.startsWith('d:')) {
        const folder = key.slice(2)
        await auth.request(
          `/api/admin/builds/${props.buildId}/files/prefix?path=${encodeURIComponent(currentPath.value + folder)}`,
          { method: 'DELETE' },
        )
      } else {
        const id = key.slice(2)
        await auth.request(`/api/admin/builds/${props.buildId}/files/${id}`, { method: 'DELETE' })
      }
    }
    selected.value = new Set()
    await loadFiles()
    emit('changed')
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally { loading.value = false }
}

/** Файл раздаётся публично по sha1 — админский маршрут для этого не нужен. */
function downloadItem() {
  const item = ctxItem.value
  if (!item || item.type !== 'file' || !item.sha1) return
  const link = document.createElement('a')
  link.href = `${auth.masterUrl.value}/files/${item.sha1}`
  link.download = item.name
  link.click()
}

function startRename() {
  closeCtx()
  if (selected.value.size !== 1) return
  const key = Array.from(selected.value)[0]
  const item = currentItems.value.find(it =>
    (it.type === 'folder' ? `d:${it.name}` : `f:${it.id}`) === key
  )
  if (!item) return
  renamingId.value = key
  renameValue.value = item.name
}

function cancelRename() {
  renamingId.value = null
  renameValue.value = ''
}

async function commitRename() {
  // Rename not supported by backend yet — just cancel
  cancelRename()
}

function copyPath() {
  closeCtx()
  const paths = Array.from(selected.value).map(key => {
    if (key.startsWith('d:')) return currentPath.value + key.slice(2) + '/'
    const it = currentItems.value.find(it => it.type === 'file' && `f:${it.id}` === key)
    return it ? currentPath.value + it.name : ''
  }).filter(Boolean)
  navigator.clipboard.writeText(paths.join('\n'))
}

async function uploadFiles(ev?: Event) {
  closeCtx()
  const inp = ev?.target as HTMLInputElement | undefined
  if (!inp?.files?.length) {
    // Trigger a hidden input
    const el = document.createElement('input')
    el.type = 'file'
    el.multiple = true
    el.onchange = () => doUpload(Array.from(el.files || []))
    el.click()
    return
  }
  await doUpload(Array.from(inp.files))
  inp.value = ''
}

async function doUpload(fileList: File[]) {
  if (!fileList.length) return
  loading.value = true
  try {
    for (const f of fileList) {
      await auth.upload(
        `/api/admin/builds/${props.buildId}/files`, 'file', f,
        { path: currentPath.value + f.name },
      )
    }
    await loadFiles()
    emit('changed')
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally { loading.value = false }
}

function createFolder() {
  closeCtx()
  const name = prompt('Folder name:')
  if (!name) return
  currentPath.value += name + '/'
}

function openEditor() {
  if (!ctxItem.value || ctxItem.value.type !== 'file' || !ctxItem.value.id) return
  editorFileId.value = ctxItem.value.id
  editorFileName.value = ctxItem.value.name
  editorFilePath.value = currentPath.value + ctxItem.value.name
  editorVisible.value = true
}

// ─── Drag & Drop ───
function onDragOver(ev: DragEvent) {
  ev.preventDefault()
  dragging.value = true
}

function onDragLeave() { dragging.value = false }

async function onDrop(ev: DragEvent) {
  ev.preventDefault()
  dragging.value = false
  const dropped = Array.from(ev.dataTransfer?.files || [])
  if (dropped.length) await doUpload(dropped)
}

// ─── Keyboard ───
function onKey(ev: KeyboardEvent) {
  if (renamingId.value) return
  if ((ev.metaKey || ev.ctrlKey) && ev.key === 'a') {
    ev.preventDefault(); selectAll()
  }
  if (ev.key === 'Delete' || ev.key === 'Backspace') {
    if (selected.value.size > 0) deleteSelected()
  }
  if (ev.key === 'Escape') { selected.value = new Set(); closeCtx() }
  if (ev.key === 'Enter' && selected.value.size === 1) {
    const key = Array.from(selected.value)[0]
    const it = currentItems.value.find(i => (i.type === 'folder' ? `d:${i.name}` : `f:${i.id}`) === key)
    if (it) openItem(it)
  }
}

onMounted(() => window.addEventListener('keydown', onKey))
onUnmounted(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div
    class="fm-dropzone flex flex-col h-[520px]"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <FileManagerToolbar
      :breadcrumb="breadcrumb"
      :search="search"
      :view-mode="viewMode"
      :loading="loading"
      @update:search="search = $event"
      @navigate="currentPath = $event; selected = new Set()"
      @back="goBack"
      @set-view="viewMode = $event"
      @new-folder="createFolder"
      @upload="uploadFiles"
      @refresh="loadFiles"
    />

    <div class="flex-1 overflow-auto noro-scroll">
      <FileManagerList
        v-if="viewMode === 'list'"
        :items="currentItems"
        :rule="rules.ruleFor"
        @toggle-rule="toggleRule"
        :selected="selected"
        :sort-key="sortKey"
        :sort-asc="sortAsc"
        :renaming-id="renamingId"
        :rename-value="renameValue"
        @select="handleSelect"
        @open="openItem"
        @context="showContext"
        @sort="toggleSort"
        @bg-context="showBgContext"
        @update:rename-value="renameValue = $event"
        @rename-commit="commitRename"
        @rename-cancel="cancelRename"
      />
      <FileManagerGrid
        v-else
        :items="currentItems"
        :rule="rules.ruleFor"
        @toggle-rule="toggleRule"
        :selected="selected"
        :renaming-id="renamingId"
        :rename-value="renameValue"
        @select="handleSelect"
        @open="openItem"
        @context="showContext"
        @bg-context="showBgContext"
        @update:rename-value="renameValue = $event"
        @rename-commit="commitRename"
        @rename-cancel="cancelRename"
      />
    </div>

    <!-- Status bar -->
    <div class="flex items-center justify-between px-3 py-1.5 border-t border-[var(--noro-border-soft)] text-xs text-[var(--noro-muted)]">
      <span>{{ currentItems.length }} items</span>
      <span v-if="selected.size">{{ selected.size }} selected</span>
    </div>

    <!-- Drop overlay -->
    <div v-if="dragging" class="fm-drop-overlay">
      <UIcon name="i-lucide-upload" class="size-6 mr-2" /> Drop files to upload
    </div>

    <!-- Context menu -->
    <FileManagerContextMenu
      v-if="ctxVisible"
      :x="ctxX"
      :y="ctxY"
      :is-folder="ctxItem?.type === 'folder'"
      :is-text="ctxIsText"
      :sync-mode="ctxItem ? rules.ruleFor(ctxItem.path).mode : undefined"
      @set-sync="setSyncFromMenu"
      :has-selection="selected.size > 0"
      @open="openItem(ctxItem!); closeCtx()"
      @edit="closeCtx(); openEditor()"
      @download="closeCtx(); downloadItem()"
      @rename="startRename"
      @copy="copyPath"
      @delete="closeCtx(); deleteSelected()"
      @new-folder="createFolder"
      @upload="uploadFiles()"
      @close="closeCtx"
    />

    <!-- Editor -->
    <FileManagerFileEditorModal
      v-if="editorVisible"
      :build-id="buildId"
      :file-id="editorFileId"
      :file-name="editorFileName"
      :file-path="editorFilePath"
      @close="editorVisible = false"
      @saved="editorVisible = false"
    />
  </div>
</template>
