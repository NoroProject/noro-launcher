<script setup lang="ts">
import type { GameServer } from "~/types/game-server";

/**
 * Пульт игрового сервера: питание, консоль, файлы, моды, бэкапы.
 *
 * Всё, что здесь есть, идёт через ServerWrapper на машине сервера. Пока он не
 * подключён, страница честно показывает, что управлять нечем.
 */
const route = useRoute();
const auth = useAuth();
await auth.loadMe();

const serverId = computed(() => String(route.params.id));
const gameServerId = computed(() => String(route.params.gsid));

const { data: servers } = await useAsyncData(`game-servers-${serverId.value}`, () =>
    auth.requestList<GameServer>(`/api/admin/servers/${serverId.value}/game-servers`),
);
const server = computed(() => servers.value?.find((s) => s.id === gameServerId.value));
/** Прочие бэкенды сборки: цели массового применения конфигов. */
const siblings = computed(
    () => (servers.value ?? []).filter((s) => s.id !== gameServerId.value && s.kind !== "proxy"),
);

const can = (perm: string) => auth.hasPermission(perm)

const wrapper = useWrapper(gameServerId.value);
const TABS = [
    { id: "console", label: "Console", icon: "i-lucide-terminal", perms: ['noro.admin.wrapper.console', 'noro.admin.wrapper.view'] },
    { id: "files", label: "Files", icon: "i-lucide-folder-tree", perms: ['noro.admin.wrapper.files'] },
    { id: "mods", label: "Mods", icon: "i-lucide-package", perms: ['noro.admin.mods.view', 'noro.admin.wrapper.files'] },
    { id: "backups", label: "Backups", icon: "i-lucide-archive", perms: ['noro.admin.wrapper.backups'] },
    { id: "restarts", label: "Restarts", icon: "i-lucide-timer-reset", perms: ['noro.admin.restarts.view'] },
] as const;

const visibleTabs = computed(() => TABS.filter(item => auth.hasAny(...item.perms)));
const tab = ref<"console" | "files" | "mods" | "backups" | "restarts">("console");

watchEffect(() => {
    if (visibleTabs.value.length && !visibleTabs.value.some(t => t.id === tab.value)) {
        tab.value = visibleTabs.value[0]!.id
    }
})
</script>

<template>
    <NoroShell
        :title="server?.name || 'GAME SERVER'"
        :subtitle="server ? `${server.mc_host}:${server.mc_port}` : 'control surface'"
    >
        <template #actions>
            <AtomButton variant="dark" icon="i-lucide-arrow-left" :to="`/admin/clients/${serverId}`">
                Pack
            </AtomButton>
        </template>

        <div class="flex flex-col gap-5 h-full min-h-0 flex-1">
            <GameserverStatusPanel
                v-if="can('noro.admin.wrapper.power')"
                class="shrink-0"
                :state="wrapper.state.value"
                :busy="wrapper.busy.value"
                @power="wrapper.power"
            />

            <nav class="flex shrink-0 flex-wrap gap-2">
                <button
                    v-for="item in visibleTabs"
                    :key="item.id"
                    type="button"
                    class="noro-chip flex items-center gap-2 px-4 py-2 text-xs font-bold uppercase tracking-wider"
                    :class="tab === item.id ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
                    @click="tab = item.id"
                >
                    <UIcon :name="item.icon" class="size-4" />
                    {{ item.label }}
                </button>
            </nav>

            <div class="flex-1 min-h-0 flex flex-col">
                <GameserverConsole
                    v-if="tab === 'console'"
                    :game-server-id="gameServerId"
                    :enabled="wrapper.connected.value"
                    @command="wrapper.command"
                    class="h-full flex flex-col min-h-0"
                />
                <GameserverFiles
                    v-else-if="tab === 'files'"
                    :game-server-id="gameServerId"
                    :enabled="wrapper.connected.value"
                    :siblings="siblings"
                    class="h-full flex flex-col min-h-0"
                />
                <GameserverMods
                    v-else-if="tab === 'mods'"
                    :game-server-id="gameServerId"
                    :server-id="serverId"
                    :enabled="wrapper.connected.value"
                    :platform="wrapper.state.value?.info?.platform ?? null"
                    class="h-full flex flex-col min-h-0"
                />
                <GameserverBackups
                    v-else-if="tab === 'backups'"
                    :game-server-id="gameServerId"
                    :enabled="wrapper.connected.value"
                    class="h-full flex flex-col min-h-0"
                />
                <!-- Расписания живут на мастере, поэтому враппер здесь не нужен:
                     завести их можно и на выключенном сервере. -->
                <GameserverRestarts
                    v-else-if="tab === 'restarts'"
                    :game-server-id="gameServerId"
                    class="h-full flex flex-col min-h-0"
                />
            </div>
        </div>
    </NoroShell>
</template>
