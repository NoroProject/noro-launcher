<script setup lang="ts">
const props = defineProps<{
  open: boolean
  ignored: string[]
  user: string[]
  saving: boolean
}>()

const emit = defineEmits<{
  'update:open': [val: boolean]
  save: [ignored: string[], user: string[]]
}>()

const ignoredText = ref('')
const userText = ref('')
const activeTab = ref<'ignored' | 'user'>('ignored')

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      ignoredText.value = props.ignored.join('\n')
      userText.value = props.user.join('\n')
    }
  },
  { immediate: true }
)

const presets = [
  { label: 'Xaero Minimap', path: 'xaerominimap/' },
  { label: 'Xaero WorldMap', path: 'xaeroworldmap/' },
  { label: 'JourneyMap', path: 'journeymap/' },
  { label: 'Options', path: 'options.txt' },
  { label: 'Shader Options', path: 'options-shaders.txt' },
  { label: 'Baritone', path: 'baritone/' },
  { label: 'Replays', path: 'replays/' },
]

function addPreset(path: string) {
  if (activeTab.value === 'ignored') {
    const lines = ignoredText.value.split('\n').map((l) => l.trim()).filter(Boolean)
    if (!lines.includes(path)) {
      lines.push(path)
      ignoredText.value = lines.join('\n')
    }
  } else {
    const lines = userText.value.split('\n').map((l) => l.trim()).filter(Boolean)
    if (!lines.includes(path)) {
      lines.push(path)
      userText.value = lines.join('\n')
    }
  }
}

function handleSave() {
  const parseLines = (text: string) =>
    text
      .split('\n')
      .map((l) => l.trim())
      .filter(Boolean)

  emit('save', parseLines(ignoredText.value), parseLines(userText.value))
}

function close() {
  emit('update:open', false)
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
      @click.self="close"
    >
      <div
        class="w-full max-w-2xl rounded-xl border border-[var(--noro-border)] bg-[var(--noro-panel)] p-6 shadow-2xl flex flex-col gap-4 max-h-[90vh]"
      >
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-[var(--noro-border-soft)] pb-4">
          <div class="flex items-center gap-3">
            <div class="flex size-9 items-center justify-center rounded-lg bg-[var(--noro-bg)] text-[var(--noro-blue)]">
              <UIcon name="i-lucide-route" class="size-5" />
            </div>
            <div>
              <h3 class="font-bold text-[var(--noro-text)] text-base">Edit Sync Rules</h3>
              <p class="text-xs text-[var(--noro-muted)]">Manually edit paths to ignore or keep player modifications</p>
            </div>
          </div>
          <button
            class="rounded-lg p-1.5 text-[var(--noro-muted)] hover:bg-[var(--noro-bg)] hover:text-[var(--noro-text)] transition"
            @click="close"
          >
            <UIcon name="i-lucide-x" class="size-5" />
          </button>
        </div>

        <!-- Tab selection -->
        <div class="flex gap-2 border-b border-[var(--noro-border-soft)] pb-2">
          <button
            class="px-4 py-2 rounded-lg text-xs font-bold transition flex items-center gap-2"
            :class="activeTab === 'ignored' ? 'bg-[var(--noro-blue)]/20 text-[var(--noro-blue)] border border-[var(--noro-blue)]/40' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
            @click="activeTab = 'ignored'"
          >
            <UIcon name="i-lucide-eye-off" class="size-4" />
            Ignored Paths (Unmanaged)
          </button>
          <button
            class="px-4 py-2 rounded-lg text-xs font-bold transition flex items-center gap-2"
            :class="activeTab === 'user' ? 'bg-[var(--noro-magenta)]/20 text-[var(--noro-magenta)] border border-[var(--noro-magenta)]/40' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
            @click="activeTab = 'user'"
          >
            <UIcon name="i-lucide-user" class="size-4" />
            User Overrides (User Managed)
          </button>
        </div>

        <!-- Quick Presets -->
        <div class="flex flex-wrap items-center gap-1.5 text-xs">
          <span class="text-[var(--noro-muted)] mr-1 font-medium">Quick Add:</span>
          <button
            v-for="p in presets"
            :key="p.path"
            class="px-2.5 py-1 rounded bg-[var(--noro-bg)] hover:bg-[var(--noro-border-soft)] text-[var(--noro-text)] border border-[var(--noro-border-soft)] text-xs font-mono transition"
            @click="addPreset(p.path)"
          >
            + {{ p.label }}
          </button>
        </div>

        <!-- Textarea editor -->
        <div class="flex-1 flex flex-col min-h-[220px]">
          <div class="mb-2 text-xs text-[var(--noro-muted)]">
            <template v-if="activeTab === 'ignored'">
              Files and folders in this list will <b>never</b> be downloaded or deleted by the launcher sync. Folders must end with <code>/</code> (e.g. <code>xaerominimap/</code>).
            </template>
            <template v-else>
              Files in this list are downloaded once on first install, then <b>never overwritten</b> by launcher sync.
            </template>
          </div>

          <textarea
            v-if="activeTab === 'ignored'"
            v-model="ignoredText"
            class="flex-1 w-full rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg)] p-3 font-mono text-xs text-[var(--noro-text)] focus:border-[var(--noro-blue)] focus:outline-none resize-none"
            placeholder="saves/&#10;xaerominimap/&#10;options.txt"
          />
          <textarea
            v-else
            v-model="userText"
            class="flex-1 w-full rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg)] p-3 font-mono text-xs text-[var(--noro-text)] focus:border-[var(--noro-magenta)] focus:outline-none resize-none"
            placeholder="options.txt&#10;config/custom_mod.json"
          />
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-end gap-3 border-t border-[var(--noro-border-soft)] pt-4">
          <button
            class="px-4 py-2 rounded-lg text-xs font-semibold text-[var(--noro-muted)] hover:text-[var(--noro-text)] hover:bg-[var(--noro-bg)] transition"
            @click="close"
          >
            Cancel
          </button>
          <button
            class="px-5 py-2 rounded-lg text-xs font-bold bg-[var(--noro-blue)] text-black hover:opacity-90 transition flex items-center gap-2 disabled:opacity-50"
            :disabled="saving"
            @click="handleSave"
          >
            <UIcon v-if="saving" name="i-lucide-loader-2" class="size-4 animate-spin" />
            <UIcon v-else name="i-lucide-check" class="size-4" />
            Save Rules
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
