/**
 * Откуда спотлайт берёт строки: разделы, серверы, игроки, команды.
 *
 * Игроки ищутся на сервере — раньше грузилась первая сотня и фильтровалась
 * локально, поэтому найти зарегистрировавшегося раньше было нельзя вовсе.
 * Разделы и серверы известны целиком, гонять их по сети на каждую букву незачем.
 */
import type { ServerRow, UserProfile } from '~/types/api'
import type { Page } from '~/types/paging'

export type SpotKind = 'nav' | 'user' | 'server' | 'action'

export interface SpotItem {
    id: string
    title: string
    subtitle?: string
    kind: SpotKind
    icon: string
    avatarUrl?: string | null
    to?: string
    /** Команда вместо перехода: сменить язык, выйти. */
    run?: () => void | Promise<void>
}

export function useSpotlightSources() {
    const auth = useAuth()
    const api = useApi()
    const notify = useNotify()
    const { t, locale, setLocale } = useT()
    const config = useRuntimeConfig()
    const { visibleItems } = useAdminNav()

    const users = ref<UserProfile[]>([])
    const servers = ref<ServerRow[]>([])
    const searching = ref(false)

    function headUrl(skinUrl?: string | null) {
        if (!skinUrl) return null
        const master = config.public.masterUrl || ''
        return `${master}/api/textures/renders?mode=flat-head&scale=8&url=${encodeURIComponent(skinUrl)}`
    }

    // Подпись — путь: так «backup» находит «Резервную копию» и в русской локали.
    const navItems = computed<SpotItem[]>(() =>
        visibleItems.value.map((n) => ({
            id: `nav-${n.to}`,
            title: n.label,
            subtitle: n.to,
            kind: 'nav',
            icon: n.icon,
            to: n.to,
        })),
    )

    const actionItems = computed<SpotItem[]>(() => {
        const next = locale.value === 'ru' ? 'en' : 'ru'
        return [
            {
                id: 'action-locale',
                title: t('spotlight-action-locale', { locale: next.toUpperCase() }),
                kind: 'action',
                icon: 'i-lucide-languages',
                run: () => setLocale(next),
            },
            {
                id: 'action-cabinet',
                title: t('nav-player-cabinet'),
                kind: 'action',
                icon: 'i-lucide-user',
                to: link.cabinet(),
            },
            {
                id: 'action-logout',
                title: t('spotlight-action-logout'),
                kind: 'action',
                icon: 'i-lucide-log-out',
                run: async () => {
                    await api.logout()
                    await navigateTo(link.login())
                },
            },
        ]
    })

    const serverItems = computed<SpotItem[]>(() =>
        servers.value.map((s) => ({
            id: `server-${s.id}`,
            title: s.name,
            subtitle: s.mc_version,
            kind: 'server',
            icon: 'i-lucide-server',
            avatarUrl: s.icon_url || null,
            to: adminLink.client(s.id),
        })),
    )

    const userItems = computed<SpotItem[]>(() =>
        users.value.map((u) => ({
            id: `user-${u.id}`,
            title: u.username,
            subtitle: [identityHandle(u) && `@${identityHandle(u)}`, u.uuid]
                .filter(Boolean)
                .join(' · '),
            kind: 'user',
            icon: 'i-lucide-user-check',
            avatarUrl: identityAvatar(u) || headUrl(u.skin_url) || null,
            to: adminLink.user(u.id),
        })),
    )

    /** Игроков ищет база: ник, Discord и UUID сразу. */
    async function searchUsers(text: string) {
        if (!auth.hasAny('noro.admin.users.view') || text.length < 2) {
            users.value = []
            return
        }
        searching.value = true
        try {
            const page = await api.request<Page<UserProfile>>(
                `/api/admin/users?limit=8&q=${encodeURIComponent(text)}`,
            )
            users.value = page.items
        } catch {
            users.value = []
        } finally {
            searching.value = false
        }
    }

    async function loadServers() {
        if (servers.value.length || !auth.hasAny('noro.admin.servers.view')) return
        // Без `.catch(() => [])`: пустой список в спотлайте неотличим от «таких
        // серверов нет», и человек делает вывод, что искать нечего.
        try {
            servers.value = await api.requestList<ServerRow>('/api/admin/servers')
        } catch (e) {
            notify.fail(e, 'Failed to load servers for search')
        }
    }

    return { navItems, actionItems, serverItems, userItems, searching, searchUsers, loadServers }
}
