<script setup lang="ts">
import type { ModrinthHit, ModrinthVersion, OptionalAddConfig } from "~/types/api";

const props = defineProps<{
    buildId: string;
    busy?: string | null;
}>();

const emit = defineEmits<{
    add: [versionId: string, optional: OptionalAddConfig | null];
}>();

const auth = useAuth();
const query = ref("");
const hits = ref<ModrinthHit[]>([]);
const selected = ref<ModrinthHit | null>(null);
const versions = ref<ModrinthVersion[]>([]);
const versionId = ref("");
const loadingSearch = ref(false);
const loadingVersions = ref(false);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
watch(query, (val) => {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (val.trim().length < 2) return;
    debounceTimer = setTimeout(search, 400);
});
const isOptional = ref(false);
const optName = ref("");
const optCategory = ref("Gameplay");
const optDesc = ref("");
const optEnabled = ref(false);
const optVisible = ref(true);
const optLimited = ref(false);

async function search() {
    if (!query.value.trim()) return;
    loadingSearch.value = true;
    try {
        const url = `/api/admin/builds/${props.buildId}/mods/search?q=${encodeURIComponent(query.value)}`;
        const result = await auth.request<{ hits: ModrinthHit[] }>(url);
        hits.value = result?.hits ?? [];
        selected.value = null;
    } finally {
        loadingSearch.value = false;
    }
}

async function selectHit(hit: ModrinthHit) {
    if (selected.value?.project_id === hit.project_id) {
        selected.value = null;
        return;
    }
    selected.value = hit;
    optName.value = hit.title;
    optDesc.value = hit.description.slice(0, 120);
    versionId.value = "";
    versions.value = [];
    loadingVersions.value = true;
    try {
        const result = await auth.request<ModrinthVersion[]>(
            `/api/admin/builds/${props.buildId}/mods/modrinth-versions?project_id=${hit.project_id}`,
        );
        versions.value = result ?? [];
        if (versions.value.length) versionId.value = versions.value[0].id;
    } finally {
        loadingVersions.value = false;
    }
}

function doAdd() {
    if (!versionId.value) return;
    emit(
        "add",
        versionId.value,
        isOptional.value
            ? {
                  name: optName.value,
                  description: optDesc.value,
                  category: optCategory.value,
                  enabled_by_default: optEnabled.value,
                  visible: optVisible.value,
                  limited: optLimited.value,
                  icon_url: selected.value?.icon_url ?? null,
                  author: selected.value?.author ?? null,
              }
            : null,
    );
}
</script>

<template>
    <div class="grid gap-3">
        <div class="flex gap-2">
            <input
                v-model="query"
                class="noro-input-sm flex-1"
                placeholder="Search Modrinth..."
                />
            <AtomButton variant="secondary"
                :loading="loadingSearch"
                icon="i-lucide-search"
                @click="search"
            />
        </div>

        <div
            v-if="hits.length"
            class="max-h-52 overflow-y-auto grid gap-0.5 custom-scrollbar"
        >
            <button
                v-for="hit in hits"
                :key="hit.project_id"
                class="flex items-center gap-2.5 rounded-lg px-2 py-1.5 text-left transition-all border"
                :class="
                    selected?.project_id === hit.project_id
                        ? 'border-[var(--noro-cream)]/30 bg-white/8'
                        : 'border-transparent hover:bg-white/4'
                "
                @click="selectHit(hit)"
            >
                <img
                    v-if="hit.icon_url"
                    :src="hit.icon_url"
                    class="size-7 rounded shrink-0"
                />
                <div
                    v-else
                    class="size-7 rounded bg-white/10 shrink-0 grid place-items-center"
                >
                    <UIcon
                        name="i-lucide-box"
                        class="size-3.5 text-[var(--noro-muted)]"
                    />
                </div>
                <div class="min-w-0">
                    <div
                        class="text-xs font-semibold text-[var(--noro-text)] truncate"
                    >
                        {{ hit.title }}
                    </div>
                    <div
                        class="text-[10px] text-[var(--noro-muted)] truncate"
                    >
                        {{ hit.description }}
                    </div>
                </div>
            </button>
        </div>

        <div
            v-if="selected"
            class="rounded-lg border border-[var(--noro-border)] bg-black/30 p-3 grid gap-3"
        >
            <div
                v-if="loadingVersions"
                class="text-xs text-[var(--noro-muted)] text-center py-2"
            >
                Loading versions...
            </div>
            <div
                v-else-if="!versions.length"
                class="text-xs text-[var(--noro-muted)] text-center py-2"
            >
                No compatible versions found
            </div>
            <label v-else class="block">
                <span class="noro-label-xs">Version</span>
                <select v-model="versionId" class="noro-input-sm mt-1 w-full">
                    <option
                        v-for="v in versions"
                        :key="v.id"
                        :value="v.id"
                    >
                        {{ v.name }}
                    </option>
                </select>
            </label>

            <div class="border-t border-[var(--noro-border)]/30 pt-2 grid gap-2">
                <UCheckbox v-model="isOptional" label="Mark as Optional" />
                <div
                    v-if="isOptional"
                    class="grid gap-2 pl-3 border-l-2 border-[var(--noro-cream)]/20"
                >
                    <div class="flex gap-2">
                        <input
                            v-model="optName"
                            class="noro-input-sm flex-1"
                            placeholder="Display name"
                        />
                        <input
                            v-model="optCategory"
                            class="noro-input-sm w-28"
                            placeholder="Category"
                        />
                    </div>
                    <textarea
                        v-model="optDesc"
                        class="noro-input-sm w-full resize-none"
                        rows="2"
                        placeholder="Description"
                    />
                    <div class="flex flex-wrap gap-3">
                        <UCheckbox
                            v-model="optEnabled"
                            label="Enabled by default"
                            class="text-xs"
                        />
                        <UCheckbox
                            v-model="optVisible"
                            label="Visible"
                            class="text-xs"
                        />
                        <UCheckbox
                            v-model="optLimited"
                            label="Restricted"
                            class="text-xs"
                        />
                    </div>
                </div>
            </div>

            <AtomButton variant="primary"
                :loading="busy === 'modrinth-add'"
                :disabled="!versionId"
                icon="i-lucide-download"
                block
                @click="doAdd"
            >
                {{ isOptional ? "Add as Optional Mod" : "Add Mod" }}
            </AtomButton>
        </div>

        <div class="pt-2 border-t border-[var(--noro-border)]/30">
            <span class="noro-label-xs mb-2 block">Direct Injection</span>
            <div class="flex gap-2">
                <input
                    v-model="versionId"
                    class="noro-input-sm flex-1 font-mono"
                    placeholder="Version ID (e.g. A4v...)"
                />
                <AtomButton variant="primary"
                    :loading="busy === 'modrinth-add'"
                    icon="i-lucide-download"
                    @click="emit('add', versionId, null)"
                >
                    Add
                </AtomButton>
            </div>
        </div>
    </div>
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
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--noro-border);
    border-radius: 6px;
    color: var(--noro-text);
    font-size: 0.8125rem;
    padding: 0.5rem 0.75rem;
    transition: all 0.2s;
}
.noro-input-sm:focus {
    outline: none;
    border-color: var(--noro-cream);
    box-shadow: 0 0 0 2px rgba(243, 231, 179, 0.1);
}
.custom-scrollbar::-webkit-scrollbar {
    width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 10px;
}
</style>
