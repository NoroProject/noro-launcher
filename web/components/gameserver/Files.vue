<script setup lang="ts">
const auth = useAuth()
const can = (perm: string) => auth.hasPermission(perm)
import type { GameServer } from "~/types/game-server";
import type { ServerEntry } from "~/types/wrapper";

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

const ctxVisible = ref(false);
const ctxX = ref(0);
const ctxY = ref(0);
const ctxEntry = ref<ServerEntry | null>(null);

function openContextMenu(e: MouseEvent, entry?: ServerEntry) {
    ctxX.value = e.clientX;
    ctxY.value = e.clientY;
    ctxEntry.value = entry || null;
    ctxVisible.value = true;
}

function onCtxOpen() {
    if (ctxEntry.value?.dir) {
        files.open(ctxEntry.value.path);
    }
    ctxVisible.value = false;
}

function onCtxEdit() {
    if (ctxEntry.value && !ctxEntry.value.dir) {
        edit(ctxEntry.value.path);
    }
    ctxVisible.value = false;
}

function onCtxDownload() {
    if (ctxEntry.value && !ctxEntry.value.dir) {
        notify.info(`Downloading ${ctxEntry.value.name}...`);
    }
    ctxVisible.value = false;
}

function onCtxCopyPath() {
    if (ctxEntry.value) {
        navigator.clipboard.writeText(ctxEntry.value.path);
        notify.ok("Path copied to clipboard");
    }
    ctxVisible.value = false;
}

function onCtxApplyToMany() {
    if (ctxEntry.value && !ctxEntry.value.dir) {
        edit(ctxEntry.value.path);
    }
    ctxVisible.value = false;
}

function onCtxNewFolder() {
    ctxVisible.value = false;
    const name = prompt("New folder name:");
    if (name && name.trim()) {
        newFolder.value = name.trim();
        createFolder();
    }
}

function onCtxRemove() {
    if (ctxEntry.value) {
        remove(ctxEntry.value.path);
    }
    ctxVisible.value = false;
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
    <section
        class="noro-panel flex flex-col h-full min-h-0 overflow-hidden"
        @contextmenu.prevent="openContextMenu($event)"
    >
        <div class="flex shrink-0 flex-wrap items-center gap-3 border-b border-[var(--noro-border)] px-4 py-3">
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
            <!-- Создание папки — запись на машине, отдельно от просмотра. -->
            <form v-if="can('noro.admin.wrapper.files')" class="flex gap-2" @submit.prevent="createFolder">
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
            class="m-4 shrink-0"
            color="warning"
            variant="subtle"
            icon="i-lucide-plug-zap"
            description="No wrapper connected — there is nothing to browse."
        />
        <UAlert
            v-else-if="files.error.value"
            class="m-4 shrink-0"
            color="error"
            variant="subtle"
            icon="i-lucide-circle-alert"
            :description="files.error.value"
        />
        <div v-else-if="files.entries.value.length" class="flex-1 min-h-0 overflow-y-auto noro-scroll divide-y divide-[var(--noro-border)]">
            <GameserverFileRow
                v-for="entry in files.entries.value"
                :key="entry.path"
                :entry="entry"
                @open="files.open(entry.path)"
                @edit="edit(entry.path)"
                @remove="remove(entry.path)"
                @contextmenu="openContextMenu($event, entry)"
            />
        </div>
        <p v-else class="p-8 text-center text-sm text-[var(--noro-muted)]">Empty directory.</p>

        <GameserverContextMenu
            v-if="ctxVisible"
            :x="ctxX"
            :y="ctxY"
            :entry="ctxEntry"
            :siblings-count="siblings.length"
            @open="onCtxOpen"
            @edit="onCtxEdit"
            @download="onCtxDownload"
            @copy-path="onCtxCopyPath"
            @apply-to-many="onCtxApplyToMany"
            @new-folder="onCtxNewFolder"
            @remove="onCtxRemove"
            @close="ctxVisible = false"
        />

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
