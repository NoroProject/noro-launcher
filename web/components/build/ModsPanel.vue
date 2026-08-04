<script setup lang="ts">
import type { OptionalAddConfig } from "~/types/api";

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

const modalOpen = ref(false);
</script>

<template>
    <section class="noro-panel overflow-hidden text-sm">
        <div
            class="flex items-center gap-3 border-b border-[var(--noro-border)] px-5 py-4"
        >
            <div
                class="flex size-10 items-center justify-center rounded-lg bg-[var(--noro-input)] text-[var(--noro-cream)]"
            >
                <UIcon name="i-lucide-search" class="size-6" />
            </div>
            <div>
                <h2 class="font-bold text-[var(--noro-text)]">Mod Database</h2>
                <p
                    class="text-[10px] text-[var(--noro-muted)] uppercase tracking-wider"
                >
                    Modrinth, CurseForge & Direct
                </p>
            </div>
        </div>

        <div class="p-4">
            <AtomButton
                icon="i-lucide-search"
                variant="secondary"
                block
                @click="modalOpen = true"
            >
                Search Mod Database
            </AtomButton>
        </div>
    </section>

    <ModDatabaseModal
        v-model="modalOpen"
        :build-id="buildId"
        :busy="busy"
        @add-modrinth="(v, o) => emit('addModrinth', v, o)"
        @add-curse-forge="(p, f) => emit('addCurseForge', p, f)"
        @add-url="(u) => emit('addUrl', u)"
    />
</template>
