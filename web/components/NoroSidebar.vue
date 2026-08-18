<script setup lang="ts">
const route = useRoute();
const auth = useAuth();
const { t } = useT();

const cabinetNav = computed(() => [
    { label: t("nav-cabinet-home"), to: "/cabinet", icon: "i-lucide-user" },
    { label: t("nav-cabinet-skin"), to: "/skin", icon: "i-lucide-shirt" },
    { label: t("nav-cabinet-punishments"), to: "/cabinet/punishments", icon: "i-lucide-alert-triangle" },
    { label: t("nav-cabinet-rules"), to: "/rules", icon: "i-lucide-book-open" },
    { label: t("nav-cabinet-apps"), to: "/cabinet/authorized-apps", icon: "i-lucide-shield-check" },
    { label: t("nav-cabinet-settings"), to: "/cabinet/settings", icon: "i-lucide-settings" },
]);

const adminNavGroups = computed(() => [
    {
        label: t("nav-group-management"),
        items: [
            { label: t("nav-admin-dashboard"), to: "/admin", icon: "i-lucide-layout-dashboard", perms: ["noro.admin.stats"] },
            { label: t("nav-admin-clients"), to: "/admin/clients", icon: "i-lucide-server", perms: ["noro.admin.servers.view"] },
            { label: t("nav-admin-mods"), to: "/admin/mods", icon: "i-lucide-library-big", perms: ["noro.admin.mods.view"] },
            { label: t("nav-admin-users"), to: "/admin/users", icon: "i-lucide-users", perms: ["noro.admin.users.view"] },
            { label: t("nav-admin-capes"), to: "/admin/capes", icon: "i-lucide-flag", perms: ["noro.admin.capes.view"] },
            { label: t("nav-admin-roles"), to: "/admin/roles", icon: "i-lucide-shield", perms: ["noro.admin.roles.view"] },
            { label: t("nav-admin-integrity"), to: "/admin/integrity", icon: "i-lucide-shield-alert", perms: ["noro.admin.integrity.view"] },
            { label: t("nav-admin-blocklist"), to: "/admin/blocklist", icon: "i-lucide-shield-x", perms: ["noro.admin.blocklist.view"] },
        ],
    },
    {
        label: t("nav-group-content"),
        items: [
            { label: t("nav-admin-news"), to: "/admin/news", icon: "i-lucide-newspaper", perms: ["noro.admin.news.view"] },
            { label: t("nav-admin-rules"), to: "/admin/rules", icon: "i-lucide-book-open", perms: ["noro.admin.rules.view"] },
            { label: t("nav-admin-reports"), to: "/admin/reports", icon: "i-lucide-flag", perms: ["noro.admin.reports.view"] },
            { label: t("nav-admin-moderation"), to: "/admin/moderation", icon: "i-lucide-message-square-warning", perms: ["noro.admin.settings.view"] },
            { label: t("nav-admin-translations"), to: "/admin/translations", icon: "i-lucide-languages", perms: ["noro.admin.translations.view"] },
            { label: t("nav-admin-wrapper"), to: "/admin/wrapper", icon: "i-lucide-package", perms: ["noro.admin.wrapper.view"] },
        ],
    },
    {
        label: t("nav-group-system"),
        items: [
            { label: t("nav-admin-launcher"), to: "/admin/launcher", icon: "i-lucide-rocket", perms: ["noro.admin.launcher.view"] },
            { label: t("nav-admin-tokens"), to: "/admin/launcher/tokens", icon: "i-lucide-key-round", perms: ["noro.admin.launcher.tokens"] },
            { label: t("nav-admin-audit"), to: "/admin/audit", icon: "i-lucide-scroll-text", perms: ["noro.admin.audit"] },
            { label: t("nav-admin-support"), to: "/admin/support", icon: "i-lucide-folder-archive", perms: ["noro.admin.support.logs"] },
            { label: t("nav-admin-settings"), to: "/admin/settings", icon: "i-lucide-sliders-horizontal", perms: ["noro.admin.settings.view"] },
        ],
    },
]);

const visibleAdminGroups = computed(() =>
    adminNavGroups.value
        .map(group => ({
            ...group,
            items: group.items.filter(item => auth.hasAny(...item.perms)),
        }))
        .filter(group => group.items.length),
);

const inAdminArea = computed(() => route.path.startsWith("/admin"));
const canAdmin = auth.canAdmin;
const allAdminItems = computed(() => visibleAdminGroups.value.flatMap(g => g.items));

const activePath = computed(() => {
    const pool = inAdminArea.value ? allAdminItems.value : cabinetNav.value;
    const m = pool.filter(item =>
        route.path === item.to || (item.to !== "/admin" && route.path.startsWith(`${item.to}/`))
    );
    return m.sort((a, b) => b.to.length - a.to.length)[0]?.to || "";
});

const initials = computed(() =>
    (auth.user.value?.username || auth.user.value?.discord_username || "N")[0].toUpperCase()
);

const navEl = ref<HTMLElement | null>(null);
const navScroll = useState('noro-sidebar-scroll', () => 0);

onMounted(() => {
    if (navEl.value) navEl.value.scrollTop = navScroll.value;
});

function rememberScroll() {
    if (navEl.value) navScroll.value = navEl.value.scrollTop;
}

function linkClass(path: string) {
    return activePath.value === path
        ? "noro-nav-link-active"
        : "noro-nav-link-idle";
}
</script>

<template>
    <aside class="flex flex-col bg-[var(--noro-sidebar)] lg:sticky lg:top-0 lg:h-screen lg:w-72">
        <div class="flex h-20 shrink-0 items-center justify-between gap-2 border-b border-[var(--noro-border)] bg-[var(--noro-bg-deep)] px-4">
            <NuxtLink to="/" class="flex items-center gap-2.5 min-w-0 transition hover:opacity-80">
                <div class="grid size-9 shrink-0 place-items-center rounded-lg">
                    <img src="/icon.png" alt="Noro" />
                </div>
                <div class="min-w-0">
                    <div class="noro-pixel text-base uppercase text-[var(--noro-cream)]">NORO</div>
                    <div class="truncate text-[10px] font-bold uppercase leading-tight tracking-wider text-[var(--noro-muted)]">{{ inAdminArea ? t("nav-admin-control") : t("nav-player-cabinet") }}</div>
                </div>
            </NuxtLink>
            <LocaleSwitch class="shrink-0" />
        </div>

        <nav ref="navEl" class="noro-scroll flex-1 overflow-y-auto px-3 py-4" @scroll.passive="rememberScroll">
            <template v-if="inAdminArea">
                <div v-for="group in visibleAdminGroups" :key="group.label" class="mb-3">
                    <div class="px-3 pb-2 pt-3 text-xs font-black uppercase tracking-wider text-[var(--noro-blue)]">{{ group.label }}</div>
                    <NuxtLink
                        v-for="item in group.items" :key="item.to" :to="item.to"
                        class="mb-1 flex items-center gap-3 rounded-lg px-3 py-3 text-sm font-bold noro-pixel uppercase tracking-wide transition-all duration-200"
                        :class="linkClass(item.to)"
                    >
                        <UIcon :name="item.icon" class="size-4 shrink-0" />
                        <span>{{ item.label }}</span>
                    </NuxtLink>
                </div>
            </template>
            <template v-else>
                <NuxtLink
                    v-for="item in cabinetNav" :key="item.to" :to="item.to"
                    class="mb-1 flex items-center gap-3 rounded-lg px-3 py-3 text-sm font-bold noro-pixel uppercase tracking-wide transition-all duration-200"
                    :class="linkClass(item.to)"
                >
                    <UIcon :name="item.icon" class="size-4 shrink-0" />
                    <span>{{ item.label }}</span>
                </NuxtLink>
            </template>
        </nav>

        <div class="shrink-0 bg-[var(--noro-bg-deep)] p-4">
            <NuxtLink
                v-if="inAdminArea || canAdmin"
                :to="inAdminArea ? '/cabinet' : '/admin'"
                class="mb-3 flex items-center gap-2 rounded-lg px-3 py-3 text-sm font-bold text-[var(--noro-muted)] transition-all duration-200 hover:scale-[1.02] hover:bg-[var(--noro-panel)] hover:text-[var(--noro-text)]"
            >
                <UIcon :name="inAdminArea ? 'i-lucide-user-round' : 'i-lucide-layout-dashboard'" class="size-3.5 shrink-0" />
                <span>{{ inAdminArea ? t("nav-switch-to-cabinet") : t("nav-switch-to-admin") }}</span>
                <UIcon name="i-lucide-arrow-right" class="ml-auto size-3 opacity-40" />
            </NuxtLink>
            <div class="noro-chip flex items-center gap-3 px-3 py-3">
                <img v-if="auth.user.value?.discord_avatar" :src="auth.user.value.discord_avatar" alt="" class="size-8 rounded-lg object-cover">
                <div v-else class="grid size-8 shrink-0 place-items-center rounded-lg bg-[var(--noro-magenta)] text-xs font-black text-[var(--noro-white)]">{{ initials }}</div>
                <div class="min-w-0 flex-1">
                    <div class="truncate text-sm font-bold text-[var(--noro-text)]">{{ auth.user.value?.username || "Guest" }}</div>
                    <div class="truncate text-xs text-[var(--noro-muted)]">@{{ auth.user.value?.discord_username || "discord" }}</div>
                </div>
                <AtomButton
                    variant="ghost"
                    size="sm"
                    icon="i-lucide-log-out"
                    class="shrink-0"
                    aria-label="Sign out"
                    @click="auth.signOut()"
                />
            </div>
        </div>
    </aside>
</template>

<style scoped>
.noro-nav-link-active {
    background: var(--noro-cream);
    color: var(--noro-on-cream);
}

.noro-nav-link-idle {
    color: var(--noro-text);
}

.noro-nav-link-idle:hover {
    background: var(--noro-panel);
    color: var(--noro-white);
    transform: scale(1.02);
}
</style>
