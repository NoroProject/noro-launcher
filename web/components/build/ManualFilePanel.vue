<script setup lang="ts">
const fileUpload = defineModel<File | null>("fileUpload", { required: true });
const filePath = defineModel<string>("filePath", { required: true });
const props = withDefaults(defineProps<{ busy?: string | null }>(), {});
defineEmits<{ upload: [] }>();
const { t } = useT();

const isDragOver = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);

function onFile(event: Event) {
    const input = event.target as HTMLInputElement;
    fileUpload.value = input.files?.[0] || null;
}

function onDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver.value = false;
    fileUpload.value = event.dataTransfer?.files?.[0] || null;
}

function onDragOver(event: DragEvent) {
    event.preventDefault();
    isDragOver.value = true;
}

function onDragLeave() {
    isDragOver.value = false;
}

function clearFile() {
    fileUpload.value = null;
}

const fileSize = computed(() => {
    if (!fileUpload.value) return "";
    const bytes = fileUpload.value.size;
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
                class="flex size-10 items-center justify-center rounded-lg bg-black/20 text-[var(--noro-blue)]"
            >
                <UIcon name="i-lucide-file-up" class="size-6" />
            </div>
            <div>
                <h2 class="font-bold text-[var(--noro-text)]">{{ t('admin-manual-title') }}</h2>
                <p
                    class="text-[10px] text-[var(--noro-muted)] uppercase tracking-wider"
                >
                    {{ t('admin-manual-subtitle') }}
                </p>
            </div>
        </div>

        <div class="p-5 grid gap-5">
            <div
                class="relative rounded-xl border-2 border-dashed px-6 py-8 text-center transition-all group"
                :class="[
                    isDragOver
                        ? 'border-[var(--noro-blue)] bg-[var(--noro-blue)]/5'
                        : 'border-[var(--noro-border)] bg-black/10 hover:border-[var(--noro-border)]/60',
                ]"
                @drop="onDrop"
                @dragover="onDragOver"
                @dragleave="onDragLeave"
            >
                <input
                    type="file"
                    class="absolute inset-0 cursor-pointer opacity-0"
                    @change="onFile"
                />

                <div class="pointer-events-none space-y-3">
                    <div v-if="!fileUpload" class="flex flex-col items-center">
                        <div
                            class="size-10 rounded-full bg-black/20 flex items-center justify-center mb-2 group-hover:scale-110 transition-transform text-[var(--noro-muted)]"
                        >
                            <UIcon name="i-lucide-plus" class="size-5" />
                        </div>
                        <p
                            class="font-bold text-[var(--noro-text)] uppercase tracking-wider text-[10px]"
                        >
                            {{ t('admin-manual-drop') }}
                        </p>
                    </div>

                    <div v-else class="flex flex-col items-center">
                        <div
                            class="size-10 rounded-full bg-[var(--noro-blue)]/20 flex items-center justify-center mb-2"
                        >
                            <UIcon
                                name="i-lucide-file-check"
                                class="size-5 text-[var(--noro-blue)]"
                            />
                        </div>
                        <p
                            class="break-words font-bold text-[var(--noro-text)] text-[11px] max-w-[200px] truncate"
                        >
                            {{ fileUpload.name }}
                        </p>
                    </div>
                </div>
            </div>

            <div>
                <span class="noro-label-xs mb-2 block">{{ t('admin-manual-path') }}</span>
                <div class="relative group">
                    <div
                        class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none"
                    >
                        <UIcon
                            name="i-lucide-folder"
                            class="size-3 text-[var(--noro-muted)]"
                        />
                    </div>
                    <input
                        v-model="filePath"
                        class="noro-input-sm pl-8 w-full"
                        placeholder="e.g. mods/utility.jar"
                    />
                </div>
            </div>

            <div class="flex gap-2">
                <AtomButton
                    icon="i-lucide-plus"
                    variant="primary"
                    :disabled="!fileUpload"
                    :loading="busy === 'file'"
                    block
                    @click="$emit('upload')"
                >
                    {{ t('admin-manual-add') }}
                </AtomButton>
                <AtomButton
                    v-if="fileUpload"
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
    border-color: var(--noro-blue);
    box-shadow: 0 0 0 2px rgba(127, 178, 255, 0.1);
}
</style>
