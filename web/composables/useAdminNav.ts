/**
 * Единый список разделов админки.
 *
 * Был в двух местах: полный с правами — в сайдбаре, укороченный и с английскими
 * подписями прямо в коде — в спотлайте. Списки разъехались ровно так, как это
 * обычно и бывает: в спотлайте не хватало восьми разделов, а те, что были,
 * показывались независимо от прав — человек нажимал и упирался в 403.
 *
 * Здесь один источник. Права проверяются один раз, подписи идут через `t()`.
 */

export interface NavItem {
    label: string
    to: string
    icon: string
    /** Достаточно любого из прав. Пусто — раздел открыт всем. */
    perms: string[]
}

export interface NavGroup {
    /** Стабильный ключ: подпись переводится, а состояние «свёрнута» — нет. */
    id: string
    label: string
    items: NavItem[]
}

export function useAdminNav() {
    const auth = useAuth()
    const { t } = useT()

    /**
     * Кабинет тоже группами — как админка.
     *
     * Правил здесь больше нет: они открыты всем и живут в шапке сайта, а в
     * личном кабинете выглядели пунктом «мои правила», которым не являются.
     */
    const cabinetGroups = computed<NavGroup[]>(() => [
        {
            id: 'cabinet-profile',
            label: t('nav-group-profile'),
            items: [
                { label: t('nav-cabinet-home'), to: link.cabinet(), icon: 'i-lucide-user', perms: [] },
                { label: t('nav-cabinet-skin'), to: link.skin(), icon: 'i-lucide-shirt', perms: [] },
                {
                    label: t('nav-cabinet-punishments'),
                    to: link.cabinetPunishments(),
                    icon: 'i-lucide-alert-triangle',
                    perms: [],
                },
            ],
        },
        {
            id: 'cabinet-account',
            label: t('nav-group-account'),
            items: [
                {
                    label: t('nav-cabinet-apps'),
                    to: link.cabinetApps(),
                    icon: 'i-lucide-shield-check',
                    perms: [],
                },
                {
                    label: t('nav-cabinet-settings'),
                    to: link.cabinetSettings(),
                    icon: 'i-lucide-settings',
                    perms: [],
                },
            ],
        },
    ])

    /** Плоский список кабинета — им подсвечивается активный раздел. */
    const cabinetNav = computed<NavItem[]>(() => cabinetGroups.value.flatMap((g) => g.items))

    const groups = computed<NavGroup[]>(() => [
        {
            id: 'management',
            label: t('nav-group-management'),
            items: [
                { label: t('nav-admin-dashboard'), to: adminLink.root(), icon: 'i-lucide-layout-dashboard', perms: ['noro.admin.stats'] },
                { label: t('nav-admin-clients'), to: adminLink.clients(), icon: 'i-lucide-server', perms: ['noro.admin.servers.view'] },
                { label: t('nav-admin-mods'), to: adminLink.mods(), icon: 'i-lucide-library-big', perms: ['noro.admin.mods.view'] },
                { label: t('nav-admin-users'), to: adminLink.users(), icon: 'i-lucide-users', perms: ['noro.admin.users.view'] },
                { label: t('nav-admin-capes'), to: adminLink.capes(), icon: 'i-lucide-flag', perms: ['noro.admin.capes.view'] },
                { label: t('nav-admin-roles'), to: adminLink.roles(), icon: 'i-lucide-shield', perms: ['noro.admin.roles.view'] },
                { label: t('nav-admin-integrity'), to: adminLink.integrity(), icon: 'i-lucide-shield-alert', perms: ['noro.admin.integrity.view'] },
                { label: t('nav-admin-blocklist'), to: adminLink.blocklist(), icon: 'i-lucide-shield-x', perms: ['noro.admin.blocklist.view'] },
            ],
        },
        {
            id: 'content',
            label: t('nav-group-content'),
            items: [
                { label: t('nav-admin-news'), to: adminLink.news(), icon: 'i-lucide-newspaper', perms: ['noro.admin.news.view'] },
                { label: t('nav-admin-rules'), to: adminLink.rules(), icon: 'i-lucide-book-open', perms: ['noro.admin.rules.view'] },
                { label: t('nav-admin-cases'), to: adminLink.cases(), icon: 'i-lucide-gavel', perms: ['noro.mod.cases.view'] },
                { label: t('nav-admin-automod'), to: adminLink.automod(), icon: 'i-lucide-shield-alert', perms: ['noro.admin.chat_filters.view'] },
                { label: t('nav-admin-moderation'), to: adminLink.moderation(), icon: 'i-lucide-message-square-warning', perms: ['noro.admin.moderation.view'] },
                { label: t('nav-admin-translations'), to: adminLink.translations(), icon: 'i-lucide-languages', perms: ['noro.admin.translations.view'] },
                { label: t('nav-admin-wrapper'), to: adminLink.wrapper(), icon: 'i-lucide-package', perms: ['noro.admin.wrapper.view'] },
            ],
        },
        {
            // Способы входа отсюда ушли во вкладку настроек: это те же поля
            // инстанса, и отдельным разделом они только удлиняли список.
            id: 'auth',
            label: t('nav-group-auth'),
            items: [
                { label: t('nav-admin-apps'), to: adminLink.apps(), icon: 'i-lucide-app-window', perms: ['noro.admin.oauth.view'] },
                { label: t('nav-admin-tokens'), to: adminLink.launcherTokens(), icon: 'i-lucide-key-round', perms: ['noro.admin.launcher.tokens', 'noro.admin.tokens.view'] },
            ],
        },
        {
            id: 'system',
            label: t('nav-group-system'),
            items: [
                { label: t('nav-admin-launcher'), to: adminLink.launcher(), icon: 'i-lucide-rocket', perms: ['noro.admin.launcher.view'] },
                { label: t('nav-admin-audit'), to: adminLink.audit(), icon: 'i-lucide-scroll-text', perms: ['noro.admin.audit'] },
                { label: t('nav-admin-support'), to: adminLink.support(), icon: 'i-lucide-folder-archive', perms: ['noro.admin.support.logs'] },
                { label: t('nav-admin-backup'), to: adminLink.backup(), icon: 'i-lucide-database-backup', perms: ['noro.admin.backup'] },
                { label: t('nav-admin-settings'), to: adminLink.settings(), icon: 'i-lucide-sliders-horizontal', perms: ['noro.admin.settings.view'] },
            ],
        },
    ])

    /**
     * Группы без разделов, до которых у человека нет прав.
     *
     * Внутри группы разделы идут по алфавиту: осмысленного порядка у них нет,
     * а тот, в котором они дописывались в код, читается как случайный — глазом
     * приходилось просматривать весь список, чтобы найти знакомое название.
     * Порядок самих групп задан вручную и остаётся: он про важность.
     */
    const visibleGroups = computed<NavGroup[]>(() =>
        groups.value
            .map((group) => ({
                ...group,
                items: group.items
                    .filter((item) => !item.perms.length || auth.hasAny(...item.perms))
                    .sort((a, b) => a.label.localeCompare(b.label)),
            }))
            .filter((group) => group.items.length),
    )

    /** Плоский список — для спотлайта, которому группы не нужны. */
    const visibleItems = computed(() => visibleGroups.value.flatMap((g) => g.items))

    return { cabinetNav, cabinetGroups, groups, visibleGroups, visibleItems }
}
