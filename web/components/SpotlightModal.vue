<script setup lang="ts">
import type { ServerRow, UserProfile } from '~/types/api'

interface SpotlightItem {
  id: string
  title: string
  subtitle?: string
  category: 'Navigation' | 'Users' | 'Servers'
  icon: string
  avatarUrl?: string | null
  to: string
}

const auth = useAuth()
const { t } = useT()
const router = useRouter()
const config = useRuntimeConfig()
const open = useState('noro-spotlight-open', () => false)

const query = ref('')
const selectedIndex = ref(0)
const inputEl = ref<HTMLInputElement | null>(null)

function headUrl(skinUrl?: string | null) {
  if (!skinUrl) return null
  const masterUrl = config.public.masterUrl || ''
  return `${masterUrl}/api/textures/renders?mode=flat-head&scale=8&url=${encodeURIComponent(skinUrl)}`
}

const staticNavItems = computed<SpotlightItem[]>(() => [
  { id: 'nav-dashboard', title: t('nav-admin-dashboard'), subtitle: 'Overview & Statistics', category: 'Navigation', icon: 'i-lucide-layout-dashboard', to: '/admin' },
  { id: 'nav-clients', title: t('nav-admin-clients'), subtitle: 'Manage builds & servers', category: 'Navigation', icon: 'i-lucide-server', to: '/admin/clients' },
  { id: 'nav-users', title: t('nav-admin-users'), subtitle: 'Users, roles, skins & bans', category: 'Navigation', icon: 'i-lucide-users', to: '/admin/users' },
  { id: 'nav-mods', title: t('nav-admin-mods'), subtitle: 'Modrinth & CurseForge search', category: 'Navigation', icon: 'i-lucide-library-big', to: '/admin/mods' },
  { id: 'nav-reports', title: t('nav-admin-reports'), subtitle: 'In-game player reports feed', category: 'Navigation', icon: 'i-lucide-flag', to: '/admin/reports' },
  { id: 'nav-automod', title: 'AutoMod Filters', subtitle: 'Chat anti-spam & filter rules', category: 'Navigation', icon: 'i-lucide-shield-alert', to: '/admin/automod' },
  { id: 'nav-moderation', title: t('nav-admin-moderation'), subtitle: 'Kick, ban & warn message templates', category: 'Navigation', icon: 'i-lucide-message-square-warning', to: '/admin/moderation' },
  { id: 'nav-rules', title: t('nav-admin-rules'), subtitle: 'Manage rule categories & items', category: 'Navigation', icon: 'i-lucide-book-open', to: '/admin/rules' },
  { id: 'nav-roles', title: t('nav-admin-roles'), subtitle: 'Access control & group permissions', category: 'Navigation', icon: 'i-lucide-shield', to: '/admin/roles' },
  { id: 'nav-capes', title: t('nav-admin-capes'), subtitle: 'Cape uploads & assignments', category: 'Navigation', icon: 'i-lucide-flag', to: '/admin/capes' },
  { id: 'nav-news', title: t('nav-admin-news'), subtitle: 'Launcher news feed', category: 'Navigation', icon: 'i-lucide-newspaper', to: '/admin/news' },
  { id: 'nav-audit', title: t('nav-admin-audit'), subtitle: 'System event trail', category: 'Navigation', icon: 'i-lucide-scroll-text', to: '/admin/audit' },
  { id: 'nav-settings', title: t('nav-admin-settings'), subtitle: 'Instance configuration', category: 'Navigation', icon: 'i-lucide-sliders-horizontal', to: '/admin/settings' },
  { id: 'nav-cabinet', title: t('nav-player-cabinet'), subtitle: 'Switch to personal cabinet', category: 'Navigation', icon: 'i-lucide-user', to: '/cabinet' },
])

const usersList = ref<UserProfile[]>([])
const serversList = ref<ServerRow[]>([])

async function loadData() {
  if (!auth.user.value) return
  try {
    const [u, s] = await Promise.all([
      auth.request<UserProfile[]>('/api/admin/users?limit=100').catch(() => []),
      auth.request<ServerRow[]>('/api/admin/servers').catch(() => []),
    ])
    usersList.value = u
    serversList.value = s
  } catch {
    // ignore
  }
}

const dynamicItems = computed<SpotlightItem[]>(() => {
  const items: SpotlightItem[] = []
  
  for (const s of serversList.value) {
    items.push({
      id: `server-${s.id}`,
      title: s.name,
      subtitle: `Pack: ${s.mc_version}`,
      category: 'Servers',
      icon: 'i-lucide-server',
      avatarUrl: s.icon_url || null,
      to: `/admin/clients/${s.id}`,
    })
  }

  for (const u of usersList.value) {
    const avatar = u.discord_avatar || headUrl(u.skin_url) || null
    items.push({
      id: `user-${u.id}`,
      title: u.username,
      subtitle: `@${u.discord_username} · ${u.uuid.slice(0, 8)}...`,
      category: 'Users',
      icon: 'i-lucide-user-check',
      avatarUrl: avatar,
      to: `/admin/users/${u.id}`,
    })
  }

  return items
})

const results = computed(() => {
  const q = query.value.trim().toLowerCase()
  const all = [...staticNavItems.value, ...dynamicItems.value]
  if (!q) return staticNavItems.value.slice(0, 8)
  return all.filter(item =>
    item.title.toLowerCase().includes(q) ||
    (item.subtitle && item.subtitle.toLowerCase().includes(q))
  ).slice(0, 15)
})

watch(results, () => {
  selectedIndex.value = 0
})

watch(open, (val) => {
  if (val) {
    query.value = ''
    selectedIndex.value = 0
    loadData()
    nextTick(() => inputEl.value?.focus())
  }
})

function navigateToItem(item: SpotlightItem) {
  open.value = false
  router.push(item.to)
}

function onGlobalKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    open.value = !open.value
  } else if (open.value) {
    if (e.key === 'Escape') {
      open.value = false
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % Math.max(1, results.value.length)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value - 1 + results.value.length) % Math.max(1, results.value.length)
    } else if (e.key === 'Enter' && results.value[selectedIndex.value]) {
      e.preventDefault()
      navigateToItem(results.value[selectedIndex.value]!)
    }
  }
}

onMounted(() => window.addEventListener('keydown', onGlobalKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onGlobalKeydown))
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-start justify-center bg-black/70 p-4 pt-20"
      @click.self="open = false"
    >
      <div class="noro-panel w-full max-w-xl overflow-hidden shadow-2xl border border-[var(--noro-border)]">
        <div class="flex items-center gap-3 border-b border-[var(--noro-border)] px-4 py-3 bg-[var(--noro-bg-deep)]">
          <UIcon name="i-lucide-search" class="size-5 text-[var(--noro-muted)] shrink-0" />
          <input
            ref="inputEl"
            v-model="query"
            type="text"
            class="w-full bg-transparent text-sm text-[var(--noro-text)] placeholder-[var(--noro-muted)] focus:outline-none"
            :placeholder="t('spotlight-placeholder')"
          >
          <kbd class="hidden sm:inline-block rounded border border-[var(--noro-border)] px-1.5 py-0.5 text-[10px] font-mono text-[var(--noro-muted)]">ESC</kbd>
        </div>

        <div v-if="results.length" class="noro-scroll max-h-96 overflow-y-auto p-2">
          <div
            v-for="(item, idx) in results"
            :key="item.id"
            class="flex items-center gap-3 rounded-lg px-3 py-2.5 text-xs font-bold transition cursor-pointer"
            :class="idx === selectedIndex ? 'bg-[var(--noro-cream)] text-[var(--noro-on-cream)]' : 'text-[var(--noro-text)] hover:bg-[var(--noro-input)]'"
            @click="navigateToItem(item)"
          >
            <!-- User / Server Avatar or Icon -->
            <img
              v-if="item.avatarUrl"
              :src="item.avatarUrl"
              class="size-7 rounded-md object-cover border border-[var(--noro-border)] bg-[var(--noro-input)] shrink-0"
              alt=""
            >
            <UIcon
              v-else
              :name="item.icon"
              class="size-4 shrink-0"
            />

            <div class="min-w-0 flex-1">
              <div class="truncate">{{ item.title }}</div>
              <div
                v-if="item.subtitle"
                class="truncate text-[10px] font-normal"
                :class="idx === selectedIndex ? 'opacity-80' : 'text-[var(--noro-muted)]'"
              >
                {{ item.subtitle }}
              </div>
            </div>
            <span
              class="px-2 py-0.5 text-[10px] rounded uppercase font-bold shrink-0"
              :class="idx === selectedIndex ? 'bg-[var(--noro-bg-deep)] text-[var(--noro-cream)]' : 'bg-[var(--noro-input)] text-[var(--noro-muted)]'"
            >
              {{ item.category }}
            </span>
          </div>
        </div>

        <div v-else class="p-6 text-center text-xs text-[var(--noro-muted)]">
          {{ t('spotlight-no-results', { query }) }}
        </div>

        <div class="flex items-center justify-between border-t border-[var(--noro-border)] px-4 py-2 bg-[var(--noro-bg-deep)] text-[10px] text-[var(--noro-muted)]">
          <span>{{ t('spotlight-nav-instructions') !== 'spotlight-nav-instructions' ? t('spotlight-nav-instructions') : 'Перемещение: ↑ ↓' }}</span>
          <span>{{ t('spotlight-open-instructions') !== 'spotlight-open-instructions' ? t('spotlight-open-instructions') : 'Cmd+K / Ctrl+K' }}</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>
