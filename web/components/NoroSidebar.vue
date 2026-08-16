<script setup lang="ts">
const route = useRoute();
const auth = useAuth();

const cabinetNav = [
    { label: "Cabinet", to: "/cabinet", icon: "i-lucide-user" },
    { label: "Skin", to: "/skin", icon: "i-lucide-shirt" },
    { label: "Apps", to: "/cabinet/authorized-apps", icon: "i-lucide-shield-check" },
    { label: "Settings", to: "/cabinet/settings", icon: "i-lucide-settings" },
];

const adminNavGroups = [
    {
        label: "Management",
        items: [
            { label: "Dashboard", to: "/admin", icon: "i-lucide-layout-dashboard" },
            { label: "Clients", to: "/admin/clients", icon: "i-lucide-server" },
            { label: "Mods", to: "/admin/mods", icon: "i-lucide-library-big" },
            { label: "Users", to: "/admin/users", icon: "i-lucide-users" },
            { label: "Capes", to: "/admin/capes", icon: "i-lucide-flag" },
            { label: "Roles", to: "/admin/roles", icon: "i-lucide-shield" },
        ],
    },
    {
        label: "Content",
        items: [
            { label: "News", to: "/admin/news", icon: "i-lucide-newspaper" },
            { label: "Translations", to: "/admin/translations", icon: "i-lucide-languages" },
            { label: "Wrapper", to: "/admin/wrapper", icon: "i-lucide-package" },
        ],
    },
    {
        label: "System",
        items: [
            { label: "Launcher", to: "/admin/launcher", icon: "i-lucide-rocket" },
            { label: "Tokens", to: "/admin/launcher/tokens", icon: "i-lucide-key-round" },
            { label: "Audit", to: "/admin/audit", icon: "i-lucide-scroll-text" },
        ],
    },
];

// Где мы сейчас — это про навигацию, а не про права.
const inAdminArea = computed(() => route.path.startsWith("/admin"));
// А это про права: тот же предикат, что и в middleware/auth.global.ts.
// Без него ссылку на админку видел каждый игрок, кликал — и молча улетал
// обратно в кабинет, потому что middleware его разворачивал.
const canAdmin = computed(() => auth.hasPermission("noro.admin.*"));
const allAdminItems = computed(() => adminNavGroups.flatMap(g => g.items));

const activePath = computed(() => {
    const pool = inAdminArea.value ? allAdminItems.value : cabinetNav;
    const m = pool.filter(item =>
        route.path === item.to || (item.to !== "/admin" && route.path.startsWith(`${item.to}/`))
    );
    return m.sort((a, b) => b.to.length - a.to.length)[0]?.to || "";
});

const initials = computed(() =>
    (auth.user.value?.username || auth.user.value?.discord_username || "N")[0].toUpperCase()
);

function linkClass(path: string) {
    return activePath.value === path
        ? "noro-nav-link-active"
        : "noro-nav-link-idle";
}
</script>

<template>
    <aside class="flex flex-col bg-[var(--noro-sidebar)] lg:sticky lg:top-0 lg:h-screen lg:w-72">
        <div class="flex h-20 shrink-0 items-center gap-3 bg-[var(--noro-bg-deep)] px-5">
            <div class="grid size-10 place-items-center rounded-lg">
                <img src="/icon.png"/>
            </div>
            <div>
                <div class="noro-pixel text-lg uppercase text-[var(--noro-cream)]">NORO</div>
                <div class="text-xs font-bold uppercase leading-tight tracking-wider text-[var(--noro-muted)]">{{ inAdminArea ? "admin control" : "player cabinet" }}</div>
            </div>
        </div>

        <nav class="noro-scroll flex-1 overflow-y-auto px-3 py-4">
            <template v-if="inAdminArea">
                <div v-for="group in adminNavGroups" :key="group.label" class="mb-3">
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
                <span>{{ inAdminArea ? "Player Cabinet" : "Admin Panel" }}</span>
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
