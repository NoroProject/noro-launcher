<script setup lang="ts">
import type { OptionalAddConfig } from "~/types/api";

const open = defineModel<boolean>({ required: true });

const props = withDefaults(
    defineProps<{
        buildId: string;
        busy?: string | null;
    }>(),
    {},
);

const emit = defineEmits<{
    addModrinth: [versionId: string, optional: OptionalAddConfig | null];
    addCurseForge: [projectId: string, fileId: string];
    addUrl: [url: string];
}>();

const activeTab = ref<"modrinth" | "curseforge" | "url">("modrinth");
const cfProjectId = ref("");
const cfFileId = ref("");
const urlValue = ref("");

function handleAddCurseForge() {
    if (!cfProjectId.value || !cfFileId.value) return;
    emit("addCurseForge", cfProjectId.value, cfFileId.value);
    cfProjectId.value = "";
    cfFileId.value = "";
}

function handleAddUrl() {
    if (!urlValue.value) return;
    emit("addUrl", urlValue.value);
    urlValue.value = "";
}
</script>

<template>
    <AtomModal v-model="open" title="MOD DATABASE" subtitle="Modrinth, CurseForge & Direct" :wide="true">
        <div
            class="p-1 -mx-5 -mt-5 mb-4 bg-[var(--noro-bg-deep)] flex gap-1"
        >
            <button
                v-for="tab in ['modrinth', 'curseforge', 'url'] as const"
                :key="tab"
                class="flex-1 px-3 py-2 rounded-md transition-all font-bold uppercase tracking-widest text-[10px]"
                :class="
                    activeTab === tab
                        ? 'bg-[var(--noro-cream)] text-[var(--noro-on-cream)]'
                        : 'text-[var(--noro-muted)] hover:bg-[var(--noro-panel)] hover:text-[var(--noro-text)]'
                "
                @click="activeTab = tab"
            >
                {{ tab }}
            </button>
        </div>

        <ModrinthSearchTab
            v-if="activeTab === 'modrinth'"
            :build-id="buildId"
            :busy="busy"
            @add="(vid, opt) => emit('addModrinth', vid, opt)"
        />

        <div
            v-if="activeTab === 'curseforge'"
            class="grid gap-4"
        >
            <div class="grid grid-cols-2 gap-3">
                <label class="block">
                    <span class="noro-label-xs">Project ID</span>
                    <input
                        v-model="cfProjectId"
                        class="noro-input-sm mt-1 w-full font-mono"
                        placeholder="e.g. 238222"
                    />
                </label>
                <label class="block">
                    <span class="noro-label-xs">File ID</span>
                    <input
                        v-model="cfFileId"
                        class="noro-input-sm mt-1 w-full font-mono"
                        placeholder="e.g. 562381"
                    />
                </label>
            </div>
            <UButton
                :loading="busy === 'curseforge-add'"
                icon="i-lucide-download-cloud"
                color="primary"
                block
                @click="handleAddCurseForge"
            >
                Add CurseForge Mod
            </UButton>
        </div>

        <div v-if="activeTab === 'url'" class="grid gap-4">
            <label class="block">
                <span class="noro-label-xs">Source URL</span>
                <input
                    v-model="urlValue"
                    class="noro-input-sm mt-1 w-full font-mono"
                    placeholder="https://example.com/mod.jar"
                />
            </label>
            <UButton
                :loading="busy === 'url'"
                icon="i-lucide-link"
                color="primary"
                block
                @click="handleAddUrl"
            >
                Add from URL
            </UButton>
        </div>
    </AtomModal>
</template>

<style scoped>
.noro-label-xs {
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--noro-muted);
}
.noro-input-sm {
    background: var(--noro-input);
    border: 0 solid transparent;
    border-radius: 8px;
    color: var(--noro-text);
    font-size: 0.8125rem;
    padding: 0.5rem 0.75rem;
    transition: all 0.2s;
}
.noro-input-sm:focus {
    outline: none;
    border-width: 2px;
    border-color: var(--noro-blue);
}
</style>
