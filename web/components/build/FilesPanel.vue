<script setup lang="ts">
import type { BuildFileRow } from "~/types/api";

const props = withDefaults(
    defineProps<{
        files?: BuildFileRow[] | null;
        busy?: string | null;
    }>(),
    {},
);
defineEmits<{ remove: [id: string] }>();

const search = ref("");
const kindFilter = ref("all");
const sortBy = ref<"path" | "kind" | "sha1">("path");
const sortDesc = ref(false);

const availableKinds = computed(() => {
    const kinds = new Set((props.files || []).map((f) => f.kind));
    return ["all", ...Array.from(kinds).sort()];
});

const fileIcon = (kind: string) => {
    const icons: Record<string, string> = {
        mod: "i-lucide-package",
        lib: "i-lucide-box",
        resource: "i-lucide-image",
        config: "i-lucide-settings",
        other: "i-lucide-file",
    };
    return icons[kind] || icons.other;
};

const fileSizeDisplay = (bytes: number) => {
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
};

const filtered = computed(() => {
    if (!Array.isArray(props.files)) return [];
    let result = [...props.files].filter((f) => {
        const matchSearch =
            f.path.toLowerCase().includes(search.value.toLowerCase()) ||
            f.kind.toLowerCase().includes(search.value.toLowerCase());
        const matchKind =
            kindFilter.value === "all" || f.kind === kindFilter.value;
        return matchSearch && matchKind;
    });
    result.sort((a, b) => {
        let aVal: string = "";
        let bVal: string = "";
        if (sortBy.value === "path") {
            aVal = a.path;
            bVal = b.path;
        } else if (sortBy.value === "kind") {
            aVal = a.kind;
            bVal = b.kind;
        } else {
            aVal = a.sha1;
            bVal = b.sha1;
        }
        return sortDesc.value
            ? bVal.localeCompare(aVal)
            : aVal.localeCompare(bVal);
    });
    return result;
});

const toggleSort = (col: "path" | "kind" | "sha1") => {
    if (sortBy.value === col) {
        sortDesc.value = !sortDesc.value;
    } else {
        sortBy.value = col;
        sortDesc.value = false;
    }
};

const sortIcon = (col: string) => {
    if (sortBy.value !== col) return "i-lucide-arrow-up-down";
    return sortDesc.value ? "i-lucide-arrow-down" : "i-lucide-arrow-up";
};
</script>

<template>
    <section class="noro-panel overflow-hidden">
        <div
            class="flex items-center justify-between border-b border-[var(--noro-border)] px-5 py-4"
        >
            <div class="flex items-center gap-3">
                <div
                    class="flex size-10 items-center justify-center rounded-lg bg-black/20 text-[var(--noro-cream)]"
                >
                    <UIcon name="i-lucide-files" class="size-6" />
                </div>
                <div>
                    <h2 class="font-bold text-[var(--noro-text)]">
                        Build Files
                    </h2>
                    <p
                        class="text-xs text-[var(--noro-muted)] uppercase tracking-wider"
                    >
                        {{ filtered?.length || 0 }} /
                        {{ files?.length || 0 }} total items
                    </p>
                </div>
            </div>
            <div class="flex items-center gap-2">
                <input
                    v-model="search"
                    type="text"
                    class="noro-input-sm w-48 md:w-64"
                    placeholder="Search files..."
                />
            </div>
        </div>

        <div
            class="flex items-center gap-1 border-b border-[var(--noro-border)] bg-black/10 px-5 py-2 overflow-x-auto"
        >
            <button
                v-for="kind in availableKinds"
                :key="kind"
                class="shrink-0 rounded px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider transition-all"
                :class="
                    kindFilter === kind
                        ? 'bg-[var(--noro-cream)] text-[var(--noro-on-cream)]'
                        : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)] hover:bg-white/5'
                "
                @click="kindFilter = kind"
            >
                {{ kind }}
            </button>
        </div>

        <div class="noro-scroll overflow-y-auto" style="max-height: 65vh">
            <table class="noro-table w-full">
                <thead class="sticky top-0 z-[1]">
                    <tr>
                        <th
                            class="cursor-pointer hover:text-[var(--noro-cream)]"
                            @click="toggleSort('path')"
                        >
                            <div class="flex items-center gap-2">
                                Path
                                <UIcon
                                    :name="sortIcon('path')"
                                    class="size-4"
                                />
                            </div>
                        </th>
                        <th
                            class="cursor-pointer hover:text-[var(--noro-cream)]"
                            @click="toggleSort('kind')"
                        >
                            <div class="flex items-center gap-2">
                                Kind
                                <UIcon
                                    :name="sortIcon('kind')"
                                    class="size-4"
                                />
                            </div>
                        </th>
                        <th>Size</th>
                        <th
                            class="cursor-pointer hover:text-[var(--noro-cream)]"
                            @click="toggleSort('sha1')"
                        >
                            <div class="flex items-center gap-2">
                                SHA1
                                <UIcon
                                    :name="sortIcon('sha1')"
                                    class="size-4"
                                />
                            </div>
                        </th>
                        <th />
                    </tr>
                </thead>
                <tbody v-if="filtered.length">
                    <tr
                        v-for="file in filtered"
                        :key="file.id"
                        class="group border-t border-[var(--noro-border)] hover:bg-white/5"
                    >
                        <td>
                            <div class="flex items-center gap-3">
                                <UIcon
                                    :name="fileIcon(file.kind)"
                                    class="size-4 text-[var(--noro-blue)]"
                                />
                                <code
                                    class="text-xs font-medium text-[var(--noro-text)]"
                                    >{{ file.path }}</code
                                >
                            </div>
                        </td>
                        <td>
                            <span
                                class="inline-flex rounded-full bg-black/30 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-[var(--noro-cream)]"
                            >
                                {{ file.kind }}
                            </span>
                        </td>
                        <td class="text-xs text-[var(--noro-muted)]">
                            {{ fileSizeDisplay(file.size) }}
                        </td>
                        <td>
                            <UTooltip :text="file.sha1" :shortcuts="[]">
                                <code
                                    class="text-[10px] text-[var(--noro-muted)]"
                                >
                                    {{ file.sha1.slice(0, 8) }}...{{
                                        file.sha1.slice(-4)
                                    }}
                                </code>
                            </UTooltip>
                        </td>
                        <td class="text-right">
                            <AtomButton
                              variant="dark"
                              icon="i-lucide-trash-2"
                              :disabled="busy === `delete-file-${file.id}`"
                              @click="$emit('remove', file.id)"
                              class="!min-h-8 !min-w-8 !px-1.5 opacity-0 transition-all group-hover:opacity-100"
                            >

                            </AtomButton>
                        </td>
                    </tr>
                </tbody>
            </table>

            <div v-if="!filtered.length" class="p-12">
                <EmptyState
                    v-if="files?.length"
                    icon="i-lucide-search-x"
                    title="No matches found"
                    text="Try adjusting your search filter"
                />
                <EmptyState
                    v-else
                    icon="i-lucide-file-plus"
                    title="No files yet"
                    text="Import files or upload manually to populate the build"
                />
            </div>
        </div>
    </section>
</template>

<style scoped>
.noro-table th {
    background: color-mix(in srgb, var(--noro-panel) 96%, black);
    padding: 0.75rem 1.25rem;
    text-align: left;
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--noro-muted);
}

.noro-table td {
    padding: 0.75rem 1.25rem;
}

.noro-input-sm {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--noro-border);
    border-radius: 6px;
    color: var(--noro-text);
    font-size: 0.8125rem;
    padding: 0.375rem 0.75rem;
    transition: all 0.2s;
}

.noro-input-sm:focus {
    outline: none;
    border-color: var(--noro-cream);
    box-shadow: 0 0 0 2px rgba(243, 231, 179, 0.1);
}
</style>
