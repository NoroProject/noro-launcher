<script setup lang="ts">
import type { PackImportKind } from "~/types/server-settings";

const { t } = useT();

const KINDS: Array<{ value: PackImportKind; label: string; hint: string }> = [
    { value: "mrpack", label: "Modrinth", hint: ".mrpack only" },
    { value: "curseforge", label: "CurseForge", hint: ".zip with manifest.json" },
    { value: "zip", label: "Folder ZIP", hint: ".zip with mods/ and config/ inside" },
];

const importFile = defineModel<File | null>("importFile", { required: true });
const importKind = defineModel<PackImportKind>("importKind", {
    required: true,
});
const props = withDefaults(defineProps<{ busy?: string | null }>(), {});
defineEmits<{ submitImport: [] }>();

const activeKind = computed(
    () => KINDS.find(k => k.value === importKind.value) || KINDS[0]!,
);

const isDragOver = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);

function onFile(event: Event) {
    const input = event.target as HTMLInputElement;
    importFile.value = input.files?.[0] || null;
}

function onDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver.value = false;
    importFile.value = event.dataTransfer?.files?.[0] || null;
}

function onDragOver(event: DragEvent) {
    event.preventDefault();
    isDragOver.value = true;
}

function onDragLeave() {
    isDragOver.value = false;
}

function clearFile() {
    importFile.value = null;
}

const fileSize = computed(() => {
    if (!importFile.value) return "";
    const bytes = importFile.value.size;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
});
</script>

<template>
    <section class="noro-panel overflow-hidden text-sm">
        <div
            class="flex items-center gap-3 border-b border-[var(--noro-border)] px-5 py-4"
        >
            <div
                class="flex size-10 items-center justify-center rounded-lg bg-black/20 text-[var(--noro-magenta)]"
            >
                <UIcon name="i-lucide-box" class="size-6" />
            </div>
            <div>
                <h2 class="font-bold text-[var(--noro-text)]">{{ t('admin-import-title') }}</h2>
                <p
                    class="text-[10px] text-[var(--noro-muted)] uppercase tracking-wider"
                >
                    {{ t('admin-import-subtitle') }}
                </p>
            </div>
        </div>

        <div class="p-5 grid gap-5">
            <div>
                <span class="noro-label-xs mb-2 block">{{ t('admin-import-format') }}</span>
                <div
                    class="flex rounded-lg bg-black/20 p-1 border border-[var(--noro-border)]"
                >
                    <button
                        v-for="kind in KINDS"
                        :key="kind.value"
                        type="button"
                        class="flex-1 px-3 py-2 rounded-md transition-all font-bold uppercase tracking-widest text-[10px]"
                        :class="
                            importKind === kind.value
                                ? 'bg-[var(--noro-magenta)] text-white shadow-lg'
                                : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'
                        "
                        @click="importKind = kind.value"
                    >
                        {{ kind.label }}
                    </button>
                </div>
            </div>

            <div
                class="relative rounded-xl border-2 border-dashed px-6 py-10 text-center transition-all group cursor-pointer"
                :class="[
                    isDragOver
                        ? 'border-[var(--noro-magenta)] bg-[var(--noro-magenta)]/5'
                        : 'border-[var(--noro-border)] bg-black/10 hover:border-[var(--noro-border)]/60',
                ]"
                @click="fileInput?.click()"
                @drop="onDrop"
                @dragover="onDragOver"
                @dragleave="onDragLeave"
            >
                <input
                    ref="fileInput"
                    type="file"
                    class="hidden"
                    @change="onFile"
                />

                <div class="pointer-events-none space-y-4">
                    <div v-if="!importFile" class="flex flex-col items-center">
                        <div
                            class="size-12 rounded-full bg-black/20 flex items-center justify-center mb-3 group-hover:scale-110 transition-transform"
                        >
                            <UIcon
                                name="i-lucide-upload-cloud"
                                class="size-6 text-[var(--noro-muted)]"
                            />
                        </div>
                        <p
                            class="font-bold text-[var(--noro-text)] uppercase tracking-wider text-[11px]"
                        >
                            {{ t('admin-import-drop') }}
                        </p>
                        <p class="text-[10px] text-[var(--noro-muted)]">
                            {{ activeKind.hint }}
                        </p>
                    </div>

                    <div v-else class="flex flex-col items-center">
                        <div
                            class="size-12 rounded-full bg-[var(--noro-green)]/20 flex items-center justify-center mb-3"
                        >
                            <UIcon
                                name="i-lucide-check-circle-2"
                                class="size-6 text-[var(--noro-green)]"
                            />
                        </div>
                        <p
                            class="break-words font-bold text-[var(--noro-text)] text-xs"
                        >
                            {{ importFile.name }}
                        </p>
                        <p
                            class="text-[10px] text-[var(--noro-muted)] uppercase font-mono"
                        >
                            {{ fileSize }}
                        </p>
                    </div>
                </div>
            </div>

            <div class="flex gap-2">
                <AtomButton
                    icon="i-lucide-sparkles"
                    variant="primary"
                    :disabled="!importFile"
                    :loading="busy === 'import'"
                    block
                    @click="$emit('submitImport')"
                >
                    {{ t('admin-import-process') }}
                </AtomButton>
                <AtomButton
                    v-if="importFile"
                    variant="dark"
                    icon="i-lucide-eraser"
                    @click="clearFile"
                />
            </div>
        </div>
    </section>
</template>

<style scoped>
.noro-label-xs {
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--noro-muted);
}
</style>
