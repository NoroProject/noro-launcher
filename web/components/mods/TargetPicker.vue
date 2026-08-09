<script setup lang="ts">
import type { InstallTarget, ModSource, OptionalDraft } from "~/types/catalog";

/**
 * Куда ставим мод. Одна установка может уехать сразу в клиентскую сборку и на
 * несколько игровых серверов — отчёт приходит построчно по каждой цели.
 */
const open = defineModel<boolean>({ required: true });

const props = defineProps<{
    source: ModSource | null;
    modName: string;
    iconUrl?: string | null;
    author?: string | null;
    serverId?: string;
    /** Сборка из контекста страницы — отмечается сразу. */
    buildId?: string;
    /** Игровой сервер из контекста страницы — тоже отмечается сразу. */
    gameServerId?: string;
}>();

const emit = defineEmits<{ installed: [] }>();

const picker = useInstallTargets(props.serverId);
const { installing, results, install } = useModInstall();

const asOptional = ref(false);
const draft = ref<OptionalDraft>(blankDraft());

function blankDraft(): OptionalDraft {
    return {
        name: props.modName,
        description: "",
        category: "Gameplay",
        enabled_by_default: false,
        visible: true,
        limited: false,
        icon_url: props.iconUrl ?? null,
        author: props.author ?? null,
    };
}

const anyBuildPicked = computed(() =>
    [...picker.picked.value].some((key) => key.startsWith("build:")),
);

async function submit() {
    if (!props.source) return;
    const list: InstallTarget[] = picker.targets((buildId) =>
        asOptional.value
            ? { kind: "build", build_id: buildId, optional: draft.value }
            : null,
    );
    if (await install(props.source, list)) {
        emit("installed");
        open.value = false;
    }
}

watch(open, async (isOpen) => {
    if (!isOpen) return;
    results.value = [];
    draft.value = blankDraft();
    await picker.loadServers();
    await picker.loadForServer();
    const preselected = new Set<string>();
    if (props.buildId) preselected.add(`build:${props.buildId}`);
    if (props.gameServerId) preselected.add(`gs:${props.gameServerId}`);
    if (preselected.size) picker.picked.value = preselected;
});
</script>

<template>
    <AtomModal v-model="open" title="INSTALL MOD" :subtitle="modName" wide>
        <div class="grid gap-5">
            <label v-if="!serverId" class="block">
                <span class="noro-label">Pack</span>
                <select v-model="picker.serverId.value" class="noro-input noro-select">
                    <option v-for="s in picker.servers.value" :key="s.id" :value="s.id">
                        {{ s.name }}
                    </option>
                </select>
            </label>

            <section class="grid gap-2">
                <span class="noro-label noro-label-inline">Client builds</span>
                <button
                    v-for="build in picker.builds.value"
                    :key="build.id"
                    type="button"
                    class="flex items-center gap-3 rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] p-3 text-left"
                    @click="picker.toggle(`build:${build.id}`)"
                >
                    <UIcon
                        :name="picker.picked.value.has(`build:${build.id}`) ? 'i-lucide-check-square' : 'i-lucide-square'"
                        class="size-4 shrink-0"
                        :class="picker.picked.value.has(`build:${build.id}`) ? 'text-[var(--noro-cream)]' : 'text-[var(--noro-muted)]'"
                    />
                    <span class="min-w-0 flex-1 truncate text-sm text-[var(--noro-text)]">
                        {{ build.version }}
                    </span>
                    <span class="shrink-0 text-xs text-[var(--noro-muted)]">
                        {{ build.modloader }} {{ build.mc_version }}
                    </span>
                    <UBadge v-if="build.published" color="success" variant="subtle" size="sm">live</UBadge>
                </button>
                <p v-if="!picker.builds.value.length" class="text-sm text-[var(--noro-muted)]">
                    This pack has no builds yet.
                </p>
            </section>

            <section v-if="anyBuildPicked" class="grid gap-3">
                <label class="flex items-center gap-2 text-sm text-[var(--noro-text)]">
                    <input v-model="asOptional" type="checkbox"> Add as an optional mod
                </label>
                <ModsOptionalForm v-if="asOptional" v-model="draft" />
            </section>

            <section class="grid gap-2">
                <div class="flex items-center justify-between">
                    <span class="noro-label noro-label-inline">Game servers</span>
                    <AtomButton
                        v-if="picker.gameServers.value.length"
                        variant="ghost"
                        size="sm"
                        icon="i-lucide-list-checks"
                        @click="picker.pickAllServers()"
                    >
                        All servers
                    </AtomButton>
                </div>
                <button
                    v-for="gs in picker.gameServers.value"
                    :key="gs.id"
                    type="button"
                    class="flex items-center gap-3 rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] p-3 text-left"
                    @click="picker.toggle(`gs:${gs.id}`)"
                >
                    <UIcon
                        :name="picker.picked.value.has(`gs:${gs.id}`) ? 'i-lucide-check-square' : 'i-lucide-square'"
                        class="size-4 shrink-0"
                        :class="picker.picked.value.has(`gs:${gs.id}`) ? 'text-[var(--noro-cream)]' : 'text-[var(--noro-muted)]'"
                    />
                    <span class="min-w-0 flex-1 truncate text-sm text-[var(--noro-text)]">{{ gs.name }}</span>
                    <span
                        class="size-2 shrink-0 rounded-full"
                        :style="{ background: gs.live ? 'var(--noro-green)' : 'var(--noro-muted)' }"
                        :title="gs.live ? 'online' : 'offline'"
                    />
                </button>
                <p v-if="!picker.gameServers.value.length" class="text-sm text-[var(--noro-muted)]">
                    No backends registered for this pack.
                </p>
            </section>

            <ul v-if="results.length" class="grid gap-1 text-xs">
                <li
                    v-for="(row, i) in results"
                    :key="i"
                    :class="row.ok ? 'text-[var(--noro-green)]' : 'text-[var(--noro-danger)]'"
                >
                    {{ row.ok ? "✓" : "✕" }} {{ row.kind }} · {{ row.error || row.path }}
                </li>
            </ul>

            <AtomButton
                variant="primary"
                icon="i-lucide-download"
                block
                :loading="installing"
                :disabled="!picker.picked.value.size"
                @click="submit"
            >
                Install to {{ picker.picked.value.size }} target(s)
            </AtomButton>
        </div>
    </AtomModal>
</template>
