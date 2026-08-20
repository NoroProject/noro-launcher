<script setup lang="ts">
export interface ModSuggestionItem {
  id: string
  title: string
  icon_url: string | null
  description: string | null
  provider: string
  project_id: string
  suggested_by_name: string | null
  status: string
  created_at: string
}

const props = defineProps<{
  items: ModSuggestionItem[]
}>()

const emit = defineEmits<{
  accept: [id: string, mode: 'optional' | 'regular', installOnServers: boolean]
  reject: [id: string]
}>()

const { t } = useT()
const open = defineModel<boolean>({ required: true })

const searchQuery = ref('')
const selectedProvider = ref<'all' | 'modrinth' | 'curseforge'>('all')
const acceptingId = ref<string | null>(null)

function modExternalUrl(provider: string, projectId: string): string {
  if (projectId?.startsWith('http://') || projectId?.startsWith('https://')) {
    return projectId
  }
  const p = provider?.toLowerCase() || ''
  if (p === 'modrinth') {
    return `https://modrinth.com/mod/${projectId}`
  }
  if (p === 'curseforge') {
    return `https://www.curseforge.com/minecraft/mc-mods/${projectId}`
  }
  return `https://modrinth.com/mod/${projectId}`
}

const filteredItems = computed(() => {
  let list = props.items || []

  if (selectedProvider.value !== 'all') {
    list = list.filter(item => item.provider?.toLowerCase() === selectedProvider.value)
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.trim().toLowerCase()
    list = list.filter(item =>
      item.title.toLowerCase().includes(q) ||
      (item.description && item.description.toLowerCase().includes(q)) ||
      (item.suggested_by_name && item.suggested_by_name.toLowerCase().includes(q)) ||
      item.provider.toLowerCase().includes(q) ||
      item.project_id.toLowerCase().includes(q)
    )
  }

  return list
})

function handleAccept(id: string, mode: 'optional' | 'regular', installOnServers: boolean) {
  acceptingId.value = id
  emit('accept', id, mode, installOnServers)
  setTimeout(() => {
    acceptingId.value = null
  }, 1000)
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 grid place-items-center bg-black/70 p-4"
      @click.self="open = false"
    >
      <div class="noro-panel w-full max-w-3xl overflow-hidden shadow-2xl border border-[var(--noro-border)]">
        <!-- Header -->
        <header class="flex items-center justify-between gap-4 border-b border-[var(--noro-border)] px-6 py-4 bg-[var(--noro-bg-deep)]">
          <div class="flex items-center gap-2">
            <UIcon name="i-lucide-sparkles" class="size-5 text-[var(--noro-amber)]" />
            <h2 class="text-lg font-black text-[var(--noro-text)]">
              {{ t('admin-mod-suggestions-title', { count: items.length }) }}
            </h2>
          </div>
          <button
            class="rounded p-1 text-[var(--noro-muted)] transition hover:bg-[var(--noro-input)] hover:text-[var(--noro-text)]"
            aria-label="Close"
            @click="open = false"
          >
            <UIcon name="i-lucide-x" class="size-5" />
          </button>
        </header>

        <!-- Search & Toolbar -->
        <div class="p-4 border-b border-[var(--noro-border)] bg-[var(--noro-bg-deep)] grid gap-3 sm:grid-cols-[1fr_auto]">
          <div class="relative flex items-center">
            <UIcon name="i-lucide-search" class="absolute left-3 size-4 text-[var(--noro-muted)] pointer-events-none" />
            <input
              v-model="searchQuery"
              type="text"
              class="noro-input w-full !pl-9 !pr-8 text-xs"
              :placeholder="t('admin-mod-suggestions-search')"
            >
            <button
              v-if="searchQuery"
              type="button"
              class="absolute right-2.5 text-[var(--noro-muted)] hover:text-[var(--noro-text)]"
              @click="searchQuery = ''"
            >
              <UIcon name="i-lucide-x" class="size-4" />
            </button>
          </div>

          <div class="flex items-center gap-1">
            <button
              type="button"
              class="noro-chip px-3 py-1.5 text-xs font-bold"
              :class="selectedProvider === 'all' ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
              @click="selectedProvider = 'all'"
            >
              All
            </button>
            <button
              type="button"
              class="noro-chip px-3 py-1.5 text-xs font-bold"
              :class="selectedProvider === 'modrinth' ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
              @click="selectedProvider = 'modrinth'"
            >
              Modrinth
            </button>
            <button
              type="button"
              class="noro-chip px-3 py-1.5 text-xs font-bold"
              :class="selectedProvider === 'curseforge' ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
              @click="selectedProvider = 'curseforge'"
            >
              CurseForge
            </button>
          </div>
        </div>

        <!-- Scrollable Suggestions List -->
        <div class="noro-scroll max-h-[60vh] overflow-y-auto p-4 space-y-3">
          <div
            v-for="item in filteredItems"
            :key="item.id"
            class="flex min-w-0 items-center justify-between gap-4 rounded-lg bg-[var(--noro-panel)] p-4 border border-[var(--noro-border)] transition hover:border-[var(--noro-cream)]/30"
          >
            <div class="flex min-w-0 flex-1 items-center gap-3">
              <img
                v-if="item.icon_url"
                :src="item.icon_url"
                class="size-11 rounded-lg object-cover shrink-0 border border-[var(--noro-border)] bg-[var(--noro-input)]"
                alt=""
              >
              <div
                v-else
                class="size-11 rounded-lg bg-[var(--noro-input)] border border-[var(--noro-border)] flex items-center justify-center shrink-0 text-[var(--noro-muted)]"
              >
                <UIcon name="i-lucide-box" class="size-6" />
              </div>

              <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-2">
                  <a
                    :href="modExternalUrl(item.provider, item.project_id)"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="font-bold text-sm text-[var(--noro-text)] hover:underline flex items-center gap-1 group"
                    :title="t('admin-mod-suggestions-open-external')"
                  >
                    <span>{{ item.title }}</span>
                    <UIcon name="i-lucide-external-link" class="size-3.5 text-[var(--noro-muted)] group-hover:text-[var(--noro-cream)] transition shrink-0" />
                  </a>
                  <span class="shrink-0 rounded bg-[var(--noro-input)] px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-[var(--noro-blue)]">
                    {{ item.suggested_by_name || "unknown" }}
                  </span>
                  <span class="shrink-0 rounded bg-[var(--noro-bg-deep)] px-2 py-0.5 text-[10px] font-mono uppercase text-[var(--noro-muted)] border border-[var(--noro-border)]">
                    {{ item.provider }}
                  </span>
                </div>
                <div class="text-xs text-[var(--noro-muted)] truncate mt-1">
                  {{ item.description || item.project_id }}
                </div>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex items-center gap-2 shrink-0">
              <UDropdownMenu
                :items="[
                  [
                    {
                      label: t('admin-mod-suggestions-accept-optional'),
                      icon: 'i-lucide-toggle-right',
                      onSelect: () => handleAccept(item.id, 'optional', false),
                    },
                    {
                      label: t('admin-mod-suggestions-accept-regular'),
                      icon: 'i-lucide-package-plus',
                      onSelect: () => handleAccept(item.id, 'regular', false),
                    },
                    {
                      label: t('admin-mod-suggestions-accept-servers'),
                      icon: 'i-lucide-server',
                      onSelect: () => handleAccept(item.id, 'regular', true),
                    },
                  ],
                ]"
              >
                <AtomButton
                  variant="primary"
                  size="sm"
                  icon="i-lucide-check"
                  :loading="acceptingId === item.id"
                >
                  {{ t('admin-mod-suggestions-accept') }}
                </AtomButton>
              </UDropdownMenu>
              <AtomButton
                variant="dark"
                size="sm"
                icon="i-lucide-x"
                @click="emit('reject', item.id)"
              >
                {{ t('admin-mod-suggestions-reject') }}
              </AtomButton>
            </div>
          </div>

          <div v-if="!filteredItems.length" class="py-12 text-center text-xs text-[var(--noro-muted)] space-y-2">
            <UIcon name="i-lucide-inbox" class="size-8 mx-auto opacity-40" />
            <div>{{ t('admin-mod-suggestions-empty') }}</div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
