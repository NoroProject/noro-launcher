<script setup lang="ts">
import type { ModVersion } from "~/types/catalog";

const props = defineProps<{ version: ModVersion }>();
defineEmits<{ install: [] }>();

const requires = computed(
    () => props.version.dependencies.filter((d) => d.kind === "required").length,
);
</script>

<template>
    <div class="flex items-center gap-3 rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] p-3 min-w-0 w-full overflow-hidden shrink-0">
        <span
            class="size-2 shrink-0 rounded-full"
            :style="{ background: channelColor(version.channel) }"
            :title="version.channel"
        />
        <div class="min-w-0 flex-1">
            <div class="truncate text-xs md:text-sm font-bold text-[var(--noro-text)]" :title="version.name">
                {{ version.name }}
            </div>
            <div class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[11px] text-[var(--noro-muted)]">
                <span>{{ version.game_versions.slice(0, 2).join(", ") || "any" }}</span>
                <span v-if="version.loaders.length" class="uppercase font-mono text-[10px] opacity-80">{{ version.loaders.join(", ") }}</span>
                <span>{{ compactBytes(version.size) }}</span>
                <span v-if="version.published">{{ relativeDate(version.published) }}</span>
                <span v-if="requires" class="text-[var(--noro-amber)] font-medium">
                    {{ requires }} dep
                </span>
            </div>
        </div>
        <AtomButton
            variant="primary"
            size="sm"
            icon="i-lucide-download"
            :disabled="!version.downloadable"
            :title="version.downloadable ? 'Install this version' : 'Author blocked third-party downloads'"
            class="shrink-0"
            @click="$emit('install')"
        />
    </div>
</template>
