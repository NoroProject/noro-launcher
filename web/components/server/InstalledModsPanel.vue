<script setup lang="ts">
import type { BuildFileRow } from "~/types/api";

const props = defineProps<{
    serverId: string;
    buildId: string | null;
    files: BuildFileRow[];
    busy?: string | null;
}>();

const emit = defineEmits<{
    remove: [fileId: string];
    upload: [file: File];
}>();

const notify = useNotify();
const auth = useAuth();
const { t } = useT();
const search = ref("");
const dropActive = ref(false);
const viewMode = ref<"grid" | "list">("grid");

const { icons: modIcons, resolveModIcons } = useModIconResolver();

function fileSizeDisplay(bytes: number) {
    if (!bytes) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

const mods = computed(() => {
    if (!Array.isArray(props.files)) return [];
    return props.files.filter((f) => {
        const isJar = f.path.toLowerCase().endsWith(".jar");
        const inMods = f.path.toLowerCase().startsWith("mods/");
        return isJar || inMods || f.kind === "mod";
    });
});

const filteredMods = computed(() => {
    if (!search.value.trim()) return mods.value;
    const q = search.value.toLowerCase().trim();
    return mods.value.filter((m) => m.path.toLowerCase().includes(q));
});

const serverIdRef = computed(() => props.serverId);
const { suggestions, accept, reject } = useModSuggestions(serverIdRef);

const acceptingId = ref<string | null>(null);
const showSuggestionsModal = ref(false);

function modExternalUrl(provider: string, projectId: string): string {
    if (projectId?.startsWith('http://') || projectId?.startsWith('https://')) return projectId;
    const p = provider?.toLowerCase() || '';
    if (p === 'modrinth') return `https://modrinth.com/mod/${projectId}`;
    if (p === 'curseforge') return `https://www.curseforge.com/minecraft/mc-mods/${projectId}`;
    return `https://modrinth.com/mod/${projectId}`;
}

async function acceptSuggestion(id: string, mode: "optional" | "regular", installOnServers: boolean) {
    acceptingId.value = id;
    try {
        await accept(id, mode, installOnServers);
        notify.ok("Mod added to the build");
    } catch (e) {
        notify.fail(e, "Failed to accept the request");
    } finally {
        acceptingId.value = null;
    }
}

async function rejectSuggestion(id: string) {
    try {
        await reject(id);
        notify.ok("Request rejected");
    } catch (e) {
        notify.fail(e, "Failed to reject the request");
    }
}

const catalogUrl = computed(() => {
    if (!props.buildId) return `/admin/mods?server=${props.serverId}`;
    return `/admin/mods?server=${props.serverId}&build=${props.buildId}`;
});

watch(
    () => mods.value.map((m) => m.sha1).join(","),
    () => resolveModIcons(mods.value, props.buildId),
    { immediate: true },
);

function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files[0]) {
        emit("upload", input.files[0]);
        input.value = "";
    }
}

function handleDrop(e: DragEvent) {
    dropActive.value = false;
    if (e.dataTransfer?.files && e.dataTransfer.files[0]) {
        const file = e.dataTransfer.files[0];
        if (file.name.endsWith(".jar")) {
            emit("upload", file);
        }
    }
}
</script>

<template>
    <div
        class="noro-panel flex min-w-0 flex-col gap-4 p-5"
        :class="{ 'ring-2 ring-[var(--noro-cream)] bg-black/40': dropActive }"
        @dragover.prevent="dropActive = true"
        @dragleave.prevent="dropActive = false"
        @drop.prevent="handleDrop"
    >
        <!-- Pending Mod Suggestions from Launcher Players -->
        <div v-if="suggestions.length" class="rounded-lg border border-[var(--noro-amber)]/40 bg-[var(--noro-amber)]/10 p-4 mb-2">
            <div class="flex items-center justify-between gap-4 mb-3">
                <h3 class="flex items-center gap-2 font-bold text-sm text-[var(--noro-amber)]">
                    <UIcon name="i-lucide-sparkles" class="size-4" />
                    {{ t('admin-mod-suggestions-title', { count: suggestions.length }) }}
                </h3>
                <AtomButton
                    variant="dark"
                    size="sm"
                    icon="i-lucide-maximize-2"
                    @click="showSuggestionsModal = true"
                >
                    {{ t('admin-mod-suggestions-expand') }}
                </AtomButton>
            </div>
            <div class="grid gap-2.5">
                <div
                    v-for="item in suggestions.slice(0, 3)"
                    :key="item.id"
                    class="flex min-w-0 items-center justify-between gap-3 rounded bg-[var(--noro-panel)] p-3 border border-[var(--noro-border)]"
                >
                    <div class="flex min-w-0 flex-1 items-center gap-3">
                        <img v-if="item.icon_url" :src="item.icon_url" class="size-9 rounded object-cover shrink-0" alt="">
                        <div v-else class="size-9 rounded bg-[var(--noro-input)] flex items-center justify-center shrink-0">
                            <UIcon name="i-lucide-box" class="size-5 text-[var(--noro-muted)]" />
                        </div>
                        <div class="min-w-0">
                            <div class="flex min-w-0 items-center gap-2">
                                <a
                                    :href="modExternalUrl(item.provider, item.project_id)"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="truncate font-bold text-sm text-[var(--noro-text)] hover:underline flex items-center gap-1 group"
                                    :title="t('admin-mod-suggestions-open-external')"
                                >
                                    <span class="truncate">{{ item.title }}</span>
                                    <UIcon name="i-lucide-external-link" class="size-3 text-[var(--noro-muted)] group-hover:text-[var(--noro-cream)] transition shrink-0" />
                                </a>
                                <span class="shrink-0 rounded bg-[var(--noro-input)] px-1.5 py-0.5 text-[10px] font-semibold text-[var(--noro-blue)]">
                                    {{ item.suggested_by_name || "unknown" }}
                                </span>
                            </div>
                            <div class="text-xs text-[var(--noro-muted)] truncate">{{ item.description || item.provider }}</div>
                        </div>
                    </div>
                    <div class="flex items-center gap-2 shrink-0">
                        <UDropdownMenu
                            :items="[
                                [
                                    {
                                        label: t('admin-mod-suggestions-accept-optional'),
                                        icon: 'i-lucide-toggle-right',
                                        onSelect: () => acceptSuggestion(item.id, 'optional', false),
                                    },
                                    {
                                        label: t('admin-mod-suggestions-accept-regular'),
                                        icon: 'i-lucide-package-plus',
                                        onSelect: () => acceptSuggestion(item.id, 'regular', false),
                                    },
                                    {
                                        label: t('admin-mod-suggestions-accept-servers'),
                                        icon: 'i-lucide-server',
                                        onSelect: () => acceptSuggestion(item.id, 'regular', true),
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
                        <AtomButton variant="dark" size="sm" icon="i-lucide-x" @click="rejectSuggestion(item.id)">
                            {{ t('admin-mod-suggestions-reject') }}
                        </AtomButton>
                    </div>
                </div>
            </div>
        </div>

        <ServerModSuggestionsModal
            v-model="showSuggestionsModal"
            :items="suggestions"
            @accept="acceptSuggestion"
            @reject="rejectSuggestion"
        />
        <!-- Header & Action Bar -->
        <div class="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--noro-border)] pb-4">
            <div class="flex items-center gap-3">
                <div class="flex size-10 items-center justify-center rounded-lg bg-[var(--noro-cream)]/10 text-[var(--noro-cream)]">
                    <UIcon name="i-lucide-puzzle" class="size-6" />
                </div>
                <div>
                    <h2 class="font-bold text-[var(--noro-text)] text-base">Assembly Mods</h2>
                    <p class="text-xs text-[var(--noro-muted)]">
                        <template v-if="search">
                            {{ filteredMods.length }} of {{ mods.length }} mods found
                        </template>
                        <template v-else>
                            {{ mods.length }} mods installed in build
                        </template>
                    </p>
                </div>
            </div>

            <div class="flex flex-wrap items-center gap-2">
                <!-- Polished Search Input -->
                <div class="relative flex items-center h-9">
                    <UIcon name="i-lucide-search" class="absolute left-3 size-4 text-[var(--noro-muted)] pointer-events-none" />
                    <input
                        v-model="search"
                        type="text"
                        placeholder="Search mods..."
                        class="noro-input-sm !pl-9 !pr-8 w-44 md:w-56 h-full text-xs rounded-lg !py-0"
                    />
                    <button
                        v-if="search"
                        type="button"
                        class="absolute right-2.5 text-[var(--noro-muted)] hover:text-[var(--noro-text)]"
                        @click="search = ''"
                    >
                        <UIcon name="i-lucide-x" class="size-3.5" />
                    </button>
                </div>

                <!-- View Mode Switcher -->
                <div class="flex items-center rounded-lg border border-[var(--noro-border)] bg-black/30 p-0.5">
                    <button
                        type="button"
                        class="p-1.5 rounded transition-colors"
                        :class="viewMode === 'grid' ? 'bg-[var(--noro-cream)]/20 text-[var(--noro-cream)]' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
                        title="Grid view"
                        @click="viewMode = 'grid'"
                    >
                        <UIcon name="i-lucide-layout-grid" class="size-4" />
                    </button>
                    <button
                        type="button"
                        class="p-1.5 rounded transition-colors"
                        :class="viewMode === 'list' ? 'bg-[var(--noro-cream)]/20 text-[var(--noro-cream)]' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
                        title="List view"
                        @click="viewMode = 'list'"
                    >
                        <UIcon name="i-lucide-list" class="size-4" />
                    </button>
                </div>

                <!-- Upload JAR Button -->
                <label class="noro-btn noro-btn-dark !min-h-[36px] !px-3 cursor-pointer text-xs uppercase font-bold flex items-center gap-1.5 rounded-lg border border-[var(--noro-border)]">
                    <UIcon name="i-lucide-upload" class="size-4" />
                    <span>Upload JAR</span>
                    <input
                        type="file"
                        accept=".jar"
                        class="hidden"
                        @change="handleFileSelect"
                    />
                </label>

                <!-- Mod Catalog Button -->
                <AtomButton
                    :to="catalogUrl"
                    variant="primary"
                    size="sm"
                    icon="i-lucide-compass"
                >
                    Mod Catalog
                </AtomButton>
            </div>
        </div>

        <!-- Mods List / Grid Content -->
        <div v-if="!buildId" class="py-12 text-center text-xs text-[var(--noro-muted)]">
            Select or create a build version above to manage installed mods.
        </div>

        <div v-else-if="!mods.length" class="py-12 text-center">
            <div class="mx-auto mb-3 grid size-12 place-items-center rounded-xl bg-[var(--noro-input)] text-[var(--noro-cream)]">
                <UIcon name="i-lucide-package" class="size-6" />
            </div>
            <h3 class="font-bold text-sm text-[var(--noro-text)]">No Mods Installed</h3>
            <p class="text-xs text-[var(--noro-muted)] max-w-sm mx-auto mt-1 mb-4">
                Browse Modrinth catalog to add compatible mods or drag & drop custom jar files here.
            </p>
            <div class="flex justify-center gap-2">
                <AtomButton :to="catalogUrl" variant="primary" icon="i-lucide-compass" size="sm">
                    Browse Modrinth Catalog
                </AtomButton>
            </div>
        </div>

        <div v-else-if="!filteredMods.length" class="py-8 text-center text-xs text-[var(--noro-muted)]">
            No mods matching "{{ search }}"
        </div>

        <!-- Grid View -->
        <div v-else-if="viewMode === 'grid'" class="grid gap-2.5 sm:grid-cols-2 lg:grid-cols-3">
            <div
                v-for="mod in filteredMods"
                :key="mod.id"
                class="group flex items-center justify-between gap-3 p-3 bg-[var(--noro-bg-subtle)] rounded-xl border border-[var(--noro-border)] hover:border-[var(--noro-cream)]/40 transition-all min-w-0"
            >
                <div class="flex items-center gap-3 min-w-0">
                    <div class="grid size-10 shrink-0 place-items-center rounded-lg bg-black/40 overflow-hidden border border-[var(--noro-border)]">
                        <img v-if="modIcons[mod.sha1]" :src="modIcons[mod.sha1]" alt="" class="size-full object-cover" />
                        <UIcon v-else name="i-lucide-package" class="size-5 text-[var(--noro-cream)]" />
                    </div>
                    <div class="min-w-0">
                        <span class="font-bold text-xs text-[var(--noro-text)] truncate block font-mono">
                            {{ cleanModTitle(mod.path) }}
                        </span>
                        <div class="flex items-center gap-2 text-[10px] text-[var(--noro-muted)] mt-0.5">
                            <span>{{ fileSizeDisplay(mod.size) }}</span>
                            <UTooltip :text="mod.sha1">
                                <span class="font-mono opacity-60">SHA1: {{ mod.sha1.slice(0, 6) }}...</span>
                            </UTooltip>
                        </div>
                    </div>
                </div>

                <AtomButton
                    variant="danger"
                    icon="i-lucide-trash-2"
                    size="sm"
                    :disabled="busy === `delete-file-${mod.id}`"
                    class="opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                    @click="emit('remove', mod.id)"
                />
            </div>
        </div>

        <!-- List View -->
        <div v-else class="flex flex-col divide-y divide-[var(--noro-border)]/40 rounded-xl border border-[var(--noro-border)] bg-black/20 overflow-hidden">
            <div
                v-for="mod in filteredMods"
                :key="mod.id"
                class="group flex items-center justify-between gap-4 px-4 py-3 hover:bg-black/40 transition-colors"
            >
                <div class="flex items-center gap-3 min-w-0 flex-1">
                    <div class="grid size-9 shrink-0 place-items-center rounded-lg bg-black/40 overflow-hidden border border-[var(--noro-border)]">
                        <img v-if="modIcons[mod.sha1]" :src="modIcons[mod.sha1]" alt="" class="size-full object-cover" />
                        <UIcon v-else name="i-lucide-package" class="size-4 text-[var(--noro-cream)]" />
                    </div>
                    <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                            <span class="font-bold text-xs text-[var(--noro-text)] truncate font-mono">
                                {{ mod.path }}
                            </span>
                        </div>
                    </div>
                </div>

                <div class="flex items-center gap-6 shrink-0">
                    <span class="text-xs text-[var(--noro-muted)] font-mono">{{ fileSizeDisplay(mod.size) }}</span>
                    <UTooltip :text="mod.sha1">
                        <span class="text-[10px] text-[var(--noro-muted)] font-mono opacity-70">
                            {{ mod.sha1.slice(0, 8) }}
                        </span>
                    </UTooltip>
                    <AtomButton
                        variant="danger"
                        icon="i-lucide-trash-2"
                        size="sm"
                        :disabled="busy === `delete-file-${mod.id}`"
                        class="opacity-0 group-hover:opacity-100 transition-opacity"
                        @click="emit('remove', mod.id)"
                    />
                </div>
            </div>
        </div>
    </div>
</template>
