<script setup lang="ts">
import type { GameServer } from "~/types/game-server";

/** Файловый менеджер серверной директории. */
const props = defineProps<{
    gameServerId: string;
    enabled: boolean;
    /** Прочие бэкенды сборки — для массового применения конфига. */
    siblings: GameServer[];
}>();

const files = useServerFiles(props.gameServerId);
const notify = useNotify();

const editorOpen = ref(false);
const editing = reactive({ path: "", content: "" });
const saving = ref(false);
const newFolder = ref("");

async function edit(path: string) {
    try {
        editing.path = path;
        editing.content = await files.read(path);
        editorOpen.value = true;
    } catch (e) {
        notify.fail(e, "Could not open the file");
    }
}

async function save(content: string) {
    saving.value = true;
    try {
        await files.write(editing.path, content);
        editing.content = content;
    } catch (e) {
        notify.fail(e, "Could not save");
    } finally {
        saving.value = false;
    }
}

async function applyToMany(content: string, targets: string[]) {
    saving.value = true;
    try {
        await files.apply(editing.path, content, targets);
        editing.content = content;
    } finally {
        saving.value = false;
    }
}

async function remove(path: string) {
    if (!confirm(`Delete ${path}? Directories go with everything inside.`)) return;
    try {
        await files.remove(path);
    } catch (e) {
        notify.fail(e, "Could not delete");
    }
}

async function createFolder() {
    const name = newFolder.value.trim();
    if (!name) return;
    try {
        await files.mkdir(name);
        newFolder.value = "";
    } catch (e) {
        notify.fail(e, "Could not create the folder");
    }
}

watch(
    () => props.enabled,
    (on) => {
        if (on) files.open(".");
    },
    { immediate: true },
);
</script>

<template>
    <section class="noro-panel overflow-hidden">
        <div class="flex flex-wrap items-center gap-3 border-b border-[var(--noro-border)] px-4 py-3">
            <AtomButton
                variant="ghost"
                size="sm"
                icon="i-lucide-corner-left-up"
                :disabled="files.path.value === '.'"
                aria-label="Up"
                @click="files.up()"
            />
            <nav class="flex min-w-0 flex-1 flex-wrap items-center gap-1 text-xs">
                <button type="button" class="text-[var(--noro-cream)]" @click="files.open('.')">
                    server
                </button>
                <template v-for="crumb in files.crumbs.value" :key="crumb.path">
                    <span class="text-[var(--noro-muted)]">/</span>
                    <button
                        type="button"
                        class="text-[var(--noro-text)] hover:text-[var(--noro-cream)]"
                        @click="files.open(crumb.path)"
                    >{{ crumb.label }}</button>
                </template>
            </nav>
            <form class="flex gap-2" @submit.prevent="createFolder">
                <input
                    v-model="newFolder"
                    class="noro-input h-9 w-40 text-xs"
                    placeholder="new folder"
                    :disabled="!enabled"
                >
                <AtomButton
                    type="submit"
                    variant="secondary"
                    size="sm"
                    icon="i-lucide-folder-plus"
                    :disabled="!enabled"
                    aria-label="Create folder"
                />
            </form>
            <AtomButton
                variant="ghost"
                size="sm"
                icon="i-lucide-refresh-cw"
                :loading="files.loading.value"
                aria-label="Refresh"
                @click="files.open(files.path.value)"
            />
        </div>

        <UAlert
            v-if="!enabled"
            class="m-4"
            color="warning"
            variant="subtle"
            icon="i-lucide-plug-zap"
            description="No wrapper connected — there is nothing to browse."
        />
        <UAlert
            v-else-if="files.error.value"
            class="m-4"
            color="error"
            variant="subtle"
            icon="i-lucide-circle-alert"
            :description="files.error.value"
        />
        <div v-else-if="files.entries.value.length" class="divide-y divide-[var(--noro-border)]">
            <GameserverFileRow
                v-for="entry in files.entries.value"
                :key="entry.path"
                :entry="entry"
                @open="files.open(entry.path)"
                @edit="edit(entry.path)"
                @remove="remove(entry.path)"
            />
        </div>
        <p v-else class="p-8 text-center text-sm text-[var(--noro-muted)]">Empty directory.</p>

        <GameserverFileEditor
            v-model="editorOpen"
            :path="editing.path"
            :content="editing.content"
            :saving="saving"
            :siblings="siblings"
            @save="save"
            @apply="applyToMany"
        />
    </section>
</template>
