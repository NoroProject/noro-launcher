<script setup lang="ts">
import type { BuildRow } from "~/types/api";

const props = defineProps<{
    builds: BuildRow[];
    selectedId: string | null;
    pending?: boolean;
}>();

const emit = defineEmits<{
    select: [id: string];
    create: [];
}>();
</script>

<template>
    <div class="noro-panel flex flex-wrap items-center justify-between gap-4 p-4">
        <div class="flex items-center gap-3 min-w-0">
            <div class="flex size-10 items-center justify-center rounded-lg bg-black/30 text-[var(--noro-cream)]">
                <UIcon name="i-lucide-layers" class="size-5" />
            </div>
            <div class="min-w-0">
                <div class="flex items-center gap-2">
                    <h3 class="font-bold text-sm text-[var(--noro-text)] uppercase tracking-wider">Active Build</h3>
                    <span v-if="builds.length" class="text-xs text-[var(--noro-muted)]">({{ builds.length }} total)</span>
                </div>
                <p class="text-xs text-[var(--noro-muted)] truncate">
                    Select assembly build version to configure files, import packs, and publish.
                </p>
            </div>
        </div>

        <div class="flex flex-wrap items-center gap-2">
            <template v-if="builds.length">
                <div class="flex items-center gap-1.5 overflow-x-auto max-w-full py-1">
                    <button
                        v-for="b in builds"
                        :key="b.id"
                        type="button"
                        class="noro-chip flex items-center gap-2 px-3 py-1.5 text-xs font-bold transition-all"
                        :class="selectedId === b.id ? 'noro-chip-on ring-1 ring-[var(--noro-cream)]/50' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
                        @click="emit('select', b.id)"
                    >
                        <span>v{{ b.version }}</span>
                        <UBadge
                            :color="b.published ? 'success' : 'neutral'"
                            variant="subtle"
                            size="xs"
                        >
                            {{ b.published ? 'live' : 'draft' }}
                        </UBadge>
                    </button>
                </div>
            </template>

            <AtomButton
                variant="primary"
                size="sm"
                icon="i-lucide-plus"
                @click="emit('create')"
            >
                New Build
            </AtomButton>
        </div>
    </div>
</template>
