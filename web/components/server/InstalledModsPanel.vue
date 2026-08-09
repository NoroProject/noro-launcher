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

const search = ref("");
const dropActive = ref(false);

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

const catalogUrl = computed(() => {
    if (!props.buildId) return `/admin/mods?server=${props.serverId}`;
    return `/admin/mods?server=${props.serverId}&build=${props.buildId}`;
});

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
        class="noro-panel flex flex-col gap-4 p-5"
        :class="{ 'ring-2 ring-[var(--noro-cream)] bg-black/40': dropActive }"
        @dragover.prevent="dropActive = true"
        @dragleave.prevent="dropActive = false"
        @drop.prevent="handleDrop"
    >
        <!-- Header & Action Bar -->
        <div class="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--noro-border)] pb-4">
            <div class="flex items-center gap-3">
                <div class="flex size-10 items-center justify-center rounded-lg bg-[var(--noro-cream)]/10 text-[var(--noro-cream)]">
                    <UIcon name="i-lucide-puzzle" class="size-6" />
                </div>
                <div>
                    <h2 class="font-bold text-[var(--noro-text)] text-base">Assembly Mods</h2>
                    <p class="text-xs text-[var(--noro-muted)]">
                        {{ filteredMods.length }} / {{ mods.length }} installed mods in build
                    </p>
                </div>
            </div>

            <div class="flex flex-wrap items-center gap-2">
                <input
                    v-model="search"
                    type="text"
                    placeholder="Search mods..."
                    class="noro-input-sm w-44 md:w-56 text-xs"
                />

                <label class="noro-btn noro-btn-dark !min-h-[36px] !px-3 cursor-pointer text-xs uppercase font-bold flex items-center gap-1.5 rounded-lg border border-[var(--noro-border)]">
                    <UIcon name="i-lucide-upload" class="size-4" />
                    <span>Upload .jar</span>
                    <input
                        type="file"
                        accept=".jar"
                        class="hidden"
                        @change="handleFileSelect"
                    />
                </label>

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

        <!-- Mods List / Grid -->
        <div v-if="!buildId" class="py-12 text-center text-xs text-[var(--noro-muted)]">
            Select or create a build version above to manage installed mods.
        </div>

        <div v-else-if="!mods.length" class="py-12 text-center">
            <div class="mx-auto mb-3 grid size-12 place-items-center rounded-xl bg-[var(--noro-input)] text-[var(--noro-cream)]">
                <UIcon name="i-lucide-package" class="size-6" />
            </div>
            <h3 class="font-bold text-sm text-[var(--noro-text)]">No Mods Installed</h3>
            <p class="text-xs text-[var(--noro-muted)] max-w-sm mx-auto mt-1 mb-4">
                Browse Modrinth catalog to add compatible mods or drag & drop custom .jar files here.
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

        <div v-else class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
            <div
                v-for="mod in filteredMods"
                :key="mod.id"
                class="group flex items-center justify-between gap-3 p-3 bg-[var(--noro-bg-subtle)] rounded-lg border border-[var(--noro-border)] hover:border-[var(--noro-cream)]/40 transition-all min-w-0"
            >
                <div class="flex items-center gap-3 min-w-0">
                    <div class="grid size-9 shrink-0 place-items-center rounded bg-black/40 text-[var(--noro-cream)]">
                        <UIcon name="i-lucide-package" class="size-5" />
                    </div>
                    <div class="min-w-0">
                        <span class="font-bold text-xs text-[var(--noro-text)] truncate block font-mono">
                            {{ mod.path.replace(/^mods\//, '') }}
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
    </div>
</template>
