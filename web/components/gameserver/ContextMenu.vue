<script setup lang="ts">
import type { ServerEntry } from "~/types/wrapper";

const props = defineProps<{
    x: number;
    y: number;
    entry?: ServerEntry | null;
    siblingsCount?: number;
}>();

const emit = defineEmits<{
    open: [];
    edit: [];
    download: [];
    copyPath: [];
    applyToMany: [];
    newFolder: [];
    remove: [];
    close: [];
}>();

const TEXT_EXTS = [
    ".properties", ".yml", ".yaml", ".json", ".toml", ".conf", ".cfg",
    ".txt", ".md", ".ini", ".log", ".sh", ".json5", ".snbt",
];

const isFolder = computed(() => props.entry?.dir ?? false);
const isText = computed(
    () =>
        !isFolder.value &&
        props.entry &&
        TEXT_EXTS.some((ext) => props.entry?.name.toLowerCase().endsWith(ext)),
);

function onClickOutside() {
    emit("close");
}

onMounted(() => {
    setTimeout(() => {
        document.addEventListener("click", onClickOutside, { once: true });
        document.addEventListener("contextmenu", onClickOutside, { once: true });
    }, 10);
});

onUnmounted(() => {
    document.removeEventListener("click", onClickOutside);
    document.removeEventListener("contextmenu", onClickOutside);
});
</script>

<template>
    <Teleport to="body">
        <div
            class="fm-ctx"
            :style="{ left: `${x}px`, top: `${y}px` }"
            @contextmenu.prevent
            @click.stop
        >
            <template v-if="entry">
                <button v-if="isFolder" type="button" class="fm-ctx-item" @click="emit('open')">
                    <UIcon name="i-lucide-folder-open" class="size-4" /> Open
                </button>
                <button v-if="isText" type="button" class="fm-ctx-item" @click="emit('edit')">
                    <UIcon name="i-lucide-pencil" class="size-4" /> Edit Text
                </button>
                <button v-if="!isFolder" type="button" class="fm-ctx-item" @click="emit('download')">
                    <UIcon name="i-lucide-download" class="size-4" /> Download
                </button>
                <button type="button" class="fm-ctx-item" @click="emit('copyPath')">
                    <UIcon name="i-lucide-copy" class="size-4" /> Copy Path
                </button>
                <button
                    v-if="isText && (siblingsCount ?? 0) > 0"
                    type="button"
                    class="fm-ctx-item"
                    @click="emit('applyToMany')"
                >
                    <UIcon name="i-lucide-layers" class="size-4" /> Sync to Other Servers
                </button>
                <div class="fm-ctx-sep" />
            </template>

            <button type="button" class="fm-ctx-item" @click="emit('newFolder')">
                <UIcon name="i-lucide-folder-plus" class="size-4" /> New Folder
            </button>

            <template v-if="entry">
                <div class="fm-ctx-sep" />
                <button type="button" class="fm-ctx-item danger" @click="emit('remove')">
                    <UIcon name="i-lucide-trash-2" class="size-4" /> Delete
                </button>
            </template>
        </div>
    </Teleport>
</template>

<style scoped>
.fm-ctx {
    position: fixed;
    z-index: 9999;
    min-width: 180px;
    background: var(--noro-panel, #0f172a);
    border: 1px solid var(--noro-border, rgba(255, 255, 255, 0.12));
    border-radius: 8px;
    padding: 6px;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-col: column;
    gap: 2px;
}

.fm-ctx-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--noro-text, #cbd5e1);
    border-radius: 6px;
    transition: all 0.15s ease;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
}

.fm-ctx-item:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
}

.fm-ctx-item.danger {
    color: #f87171;
}

.fm-ctx-item.danger:hover {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
}

.fm-ctx-sep {
    height: 1px;
    background: var(--noro-border, rgba(255, 255, 255, 0.1));
    margin: 4px 0;
}
</style>
