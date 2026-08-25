<script setup lang="ts">
const route = useRoute();
const auth = useAuth();
const { t } = useT();

// Разделы — из общего источника: тот же список видит спотлайт.
const { cabinetNav, cabinetGroups, visibleGroups: visibleAdminGroups, visibleItems: allAdminItems } = useAdminNav();

const inAdminArea = computed(() => route.path.startsWith("/admin"));

// Кабинет и админка рисуются одним и тем же списком групп: разметка у них
// общая, и расходиться ей незачем.
const navGroups = computed(() => (inAdminArea.value ? visibleAdminGroups.value : cabinetGroups.value));
const canAdmin = auth.canAdmin;

const activePath = computed(() => {
    const pool = inAdminArea.value ? allAdminItems.value : cabinetNav.value;
    const m = pool.filter(item =>
        route.path === item.to || (item.to !== "/admin" && route.path.startsWith(`${item.to}/`))
    );
    return m.sort((a, b) => b.to.length - a.to.length)[0]?.to || "";
});

const initials = computed(() =>
    (auth.user.value?.username || identityHandle(auth.user.value) || "N")[0].toUpperCase()
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

// Свёрнутые категории. В куке, а не в useState: выбор должен пережить
// перезагрузку страницы — иначе сворачивать бессмысленно. Ключи стабильные,
// подписи переводятся.
const collapsed = useCookie<string[]>("noro_nav_collapsed", {
    sameSite: "lax",
    default: () => [],
});

function toggleGroup(id: string) {
    collapsed.value = collapsed.value.includes(id)
        ? collapsed.value.filter(x => x !== id)
        : [...collapsed.value, id];
}

const isCollapsed = (id: string) => collapsed.value.includes(id);

/** Свёрнутая группа с текущим разделом внутри подсвечивается — иначе непонятно, где ты. */
const holdsActive = (group: { items: { to: string }[] }) =>
    group.items.some(item => item.to === activePath.value);
const openSpotlight = useState('noro-spotlight-open', () => false);
</script>

<template>
    <!-- Граница справа, а не разница заливок: фон сайдбара и фон страницы
         отличаются на несколько пунктов яркости, и колонка расплывалась —
         особенно снизу, где подвал был ещё и третьего оттенка. -->
    <aside class="flex flex-col border-r border-[var(--noro-border)] bg-[var(--noro-sidebar)] lg:sticky lg:top-0 lg:h-screen lg:w-72">
        <div class="flex h-20 shrink-0 items-center justify-between gap-2 border-b border-[var(--noro-border)] bg-[var(--noro-bg-deep)] px-4">
            <NuxtLink :to="link.home()" class="flex items-center gap-2.5 min-w-0 transition hover:opacity-80">
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

        <div v-if="inAdminArea" class="px-3 pt-3">
            <button
                type="button"
                class="flex w-full items-center gap-2.5 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] px-3 py-2 text-xs text-[var(--noro-muted)] transition hover:border-[var(--noro-cream)] hover:text-[var(--noro-text)]"
                @click="openSpotlight = true"
            >
                <UIcon name="i-lucide-search" class="size-4 shrink-0 text-[var(--noro-blue)]" />
                <span class="truncate">{{ t('spotlight-quick-search') }}</span>
                <kbd class="ml-auto rounded border border-[var(--noro-border)] px-1.5 py-0.5 text-[10px] font-mono">⌘K</kbd>
            </button>
        </div>

        <nav ref="navEl" class="noro-scroll flex-1 overflow-y-auto px-3 py-4" @scroll.passive="rememberScroll">
            <div v-for="group in navGroups" :key="group.id" class="mb-3">
                    <button
                        type="button"
                        class="flex w-full items-center gap-2 rounded-lg px-3 pb-2 pt-3 text-xs font-black uppercase tracking-wider transition hover:text-[var(--noro-cream)]"
                        :class="isCollapsed(group.id) && holdsActive(group) ? 'text-[var(--noro-cream)]' : 'text-[var(--noro-blue)]'"
                        :aria-expanded="!isCollapsed(group.id)"
                        @click="toggleGroup(group.id)"
                    >
                        <span class="truncate">{{ group.label }}</span>
                        <span class="ml-auto flex items-center gap-1.5 opacity-60">
                            <span v-if="isCollapsed(group.id)" class="text-[10px] font-bold">{{ group.items.length }}</span>
                            <UIcon
                                name="i-lucide-chevron-down"
                                class="size-3.5 transition-transform duration-200"
                                :class="isCollapsed(group.id) ? '-rotate-90' : ''"
                            />
                        </span>
                    </button>
                    <NuxtLink
                        v-for="item in (isCollapsed(group.id) ? [] : group.items)" :key="item.to" :to="item.to"
                        class="mb-1 flex items-center gap-3 rounded-lg px-3 py-3 text-sm font-bold noro-pixel uppercase tracking-wide transition-all duration-200"
                        :class="linkClass(item.to)"
                    >
                        <UIcon :name="item.icon" class="size-4 shrink-0" />
                        <span>{{ item.label }}</span>
                    </NuxtLink>
                </div>
        </nav>

        <div class="shrink-0 border-t border-[var(--noro-border)] bg-[var(--noro-sidebar)] p-4">
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
                <img v-if="identityAvatar(auth.user.value)" :src="identityAvatar(auth.user.value)!" alt="" class="size-8 rounded-lg object-cover">
                <div v-else class="grid size-8 shrink-0 place-items-center rounded-lg bg-[var(--noro-magenta)] text-xs font-black text-[var(--noro-white)]">{{ initials }}</div>
                <div class="min-w-0 flex-1">
                    <div class="truncate text-sm font-bold text-[var(--noro-text)]">{{ auth.user.value?.username || "Guest" }}</div>
                    <div class="truncate text-xs text-[var(--noro-muted)]">{{ identityHandle(auth.user.value) ? `@${identityHandle(auth.user.value)}` : auth.user.value?.username }}</div>
                </div>
                <!-- Выход красным и здесь: одно действие — один цвет, иначе
                     кремовая иконка читается как обычный переход по разделам. -->
                <AtomButton
                    variant="danger-soft"
                    size="sm"
                    icon="i-lucide-log-out"
                    class="shrink-0"
                    aria-label="Sign out"
                    @click="auth.signOut()"
                />
            </div>
        </div>
        <SpotlightModal />
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
