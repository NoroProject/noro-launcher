<script setup lang="ts">
import type { ServerEntry } from "~/types/wrapper";

/** Строка файлового списка. Текстовое открывается редактором, прочее — нет. */
const props = defineProps<{ entry: ServerEntry }>();
defineEmits<{ open: []; edit: []; remove: []; contextmenu: [event: MouseEvent] }>();

/** Расширения, которые есть смысл править как текст. */
const TEXT = [
    ".properties", ".yml", ".yaml", ".json", ".toml", ".conf", ".cfg",
    ".txt", ".md", ".ini", ".log", ".sh", ".json5", ".snbt",
];

const editable = computed(
    () => !props.entry.dir && TEXT.some((ext) => props.entry.name.toLowerCase().endsWith(ext)),
);
</script>

<template>
    <div
        class="flex items-center gap-3 px-4 py-3 transition hover:bg-white/5 cursor-pointer select-none"
        @contextmenu.prevent="$emit('contextmenu', $event)"
    >
        <UIcon
            :name="entry.dir ? 'i-lucide-folder' : editable ? 'i-lucide-file-text' : 'i-lucide-file'"
            class="size-4 shrink-0"
            :class="entry.dir ? 'text-[var(--noro-cream)]' : 'text-[var(--noro-muted)]'"
        />
        <button
            type="button"
            class="min-w-0 flex-1 truncate text-left text-sm text-[var(--noro-text)]"
            :class="{ 'cursor-default': !entry.dir && !editable }"
            @click="entry.dir ? $emit('open') : editable && $emit('edit')"
        >
            {{ entry.name }}
        </button>
        <span class="shrink-0 text-xs text-[var(--noro-muted)]">
            {{ entry.dir ? "—" : compactBytes(entry.size) }}
        </span>
        <span class="hidden shrink-0 text-xs text-[var(--noro-muted)] sm:block">
            {{ relativeDate(new Date(entry.modified).toISOString()) }}
        </span>
        <AtomButton
            v-if="editable"
            variant="ghost"
            size="sm"
            icon="i-lucide-pencil"
            aria-label="Edit"
            @click="$emit('edit')"
        />
        <AtomButton
            variant="ghost"
            size="sm"
            icon="i-lucide-trash-2"
            aria-label="Delete"
            @click="$emit('remove')"
        />
    </div>
</template>
