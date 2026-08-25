<script setup lang="ts">
/**
 * Выбор архива для восстановления.
 *
 * С перетаскиванием: архив лежит в загрузках рядом с браузером, и тащить его
 * мышью быстрее, чем искать в диалоге.
 */
const props = defineProps<{ disabled?: boolean }>()
const file = defineModel<File | null>({ required: true })
const { t } = useT()

const dragging = ref(false)

const size = computed(() => {
    if (!file.value) return ''
    const mb = file.value.size / 1_048_576
    return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${mb.toFixed(1)} MB`
})

function take(picked: File | undefined) {
    dragging.value = false
    if (!props.disabled && picked) file.value = picked
}
</script>

<template>
    <label
        class="group relative grid cursor-pointer place-items-center gap-3 rounded-[var(--noro-r-sm)] border-2 border-dashed p-6 text-center transition-all duration-150"
        :class="[
            dragging
                ? 'border-[var(--noro-cream)] bg-[color-mix(in_srgb,var(--noro-cream)_10%,var(--noro-input))]'
                : 'border-[var(--noro-border)] bg-[var(--noro-input)] hover:border-[var(--noro-cream)]/50',
            disabled ? 'pointer-events-none opacity-50' : '',
        ]"
        @dragover.prevent="dragging = true"
        @dragleave.prevent="dragging = false"
        @drop.prevent="take($event.dataTransfer?.files?.[0])"
    >
        <UIcon
            :name="file ? 'i-lucide-file-archive' : 'i-lucide-upload-cloud'"
            class="size-8 text-[var(--noro-cream)] transition-transform group-hover:scale-110"
        />
        <span class="grid gap-1">
            <span class="text-xs font-bold text-[var(--noro-text)]">
                {{ file ? file.name : t('admin-backup-dropzone') }}
            </span>
            <span class="text-[10px] text-[var(--noro-muted)]">
                {{ file ? size : t('admin-backup-dropzone-hint') }}
            </span>
        </span>

        <input
            class="hidden"
            type="file"
            accept=".gz,.tgz,application/gzip"
            :disabled="disabled"
            @change="take(($event.target as HTMLInputElement).files?.[0])"
        />
    </label>
</template>
