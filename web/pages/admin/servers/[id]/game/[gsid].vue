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
    auth.request<GameServer[]>(`/api/admin/servers/${serverId.value}/game-servers`),
);
const server = computed(() => servers.value?.find((s) => s.id === gameServerId.value));
/** Прочие бэкенды сборки: цели массового применения конфигов. */
const siblings = computed(
    () => (servers.value ?? []).filter((s) => s.id !== gameServerId.value && s.kind !== "proxy"),
);

const wrapper = useWrapper(gameServerId.value);
const tab = ref<"console" | "files" | "mods" | "backups">("console");
const TABS = [
    { id: "console", label: "Console", icon: "i-lucide-terminal" },
    { id: "files", label: "Files", icon: "i-lucide-folder-tree" },
    { id: "mods", label: "Mods", icon: "i-lucide-package" },
    { id: "backups", label: "Backups", icon: "i-lucide-archive" },
] as const;
</script>

<template>
    <NoroShell
        :title="server?.name || 'GAME SERVER'"
        :subtitle="server ? `${server.mc_host}:${server.mc_port}` : 'control surface'"
    >
        <template #actions>
            <AtomButton variant="dark" icon="i-lucide-arrow-left" :to="`/admin/servers/${serverId}`">
                Pack
            </AtomButton>
        </template>

        <div class="flex flex-col gap-5 h-full min-h-0 flex-1">
            <GameserverStatusPanel
                class="shrink-0"
                :state="wrapper.state.value"
                :busy="wrapper.busy.value"
                @power="wrapper.power"
            />

            <nav class="flex shrink-0 flex-wrap gap-2">
                <button
                    v-for="item in TABS"
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
            </div>
        </div>
    </NoroShell>
</template>
