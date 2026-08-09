<script setup lang="ts">
import type { ModHit, ModProject, ModVersion } from "~/types/catalog";

/** Панель проекта: версии, описание, галерея. Правая колонка браузера. */
defineProps<{
    hit: ModHit;
    project: ModProject | null;
    versions: ModVersion[];
    loading?: boolean;
    loadingVersions?: boolean;
    compatibleOnly: boolean;
    hasContext: boolean;
}>();

const emit = defineEmits<{
    close: [];
    install: [version: ModVersion];
    "update:compatibleOnly": [value: boolean];
}>();

const tab = ref<"versions" | "about" | "gallery">("versions");
const TABS = ["versions", "about", "gallery"] as const;
</script>

<template>
    <section class="noro-panel grid content-start gap-4 p-4">
        <header class="flex items-start gap-3">
            <img
                v-if="hit.icon_url"
                :src="hit.icon_url"
                alt=""
                class="size-12 shrink-0 rounded-[var(--noro-r-sm)] object-cover"
            >
            <div class="min-w-0 flex-1">
                <h2 class="truncate font-bold text-[var(--noro-cream)]">{{ hit.title }}</h2>
                <p class="mt-1 truncate text-xs text-[var(--noro-muted)]">
                    {{ hit.author }} · {{ compactNumber(hit.downloads) }} downloads
                </p>
            </div>
            <AtomButton variant="ghost" size="sm" icon="i-lucide-x" aria-label="Close" @click="emit('close')" />
        </header>

        <div class="flex flex-wrap gap-2">
            <AtomButton
                v-if="hit.page_url"
                variant="ghost"
                size="sm"
                icon="i-lucide-external-link"
                :href="hit.page_url"
            >
                Page
            </AtomButton>
            <AtomButton
                v-if="project?.source_url"
                variant="ghost"
                size="sm"
                icon="i-lucide-code"
                :href="project.source_url"
            >
                Source
            </AtomButton>
            <AtomButton
                v-if="project?.issues_url"
                variant="ghost"
                size="sm"
                icon="i-lucide-bug"
                :href="project.issues_url"
            >
                Issues
            </AtomButton>
        </div>

        <nav class="flex gap-1 rounded-[var(--noro-r-sm)] bg-[var(--noro-bg-deep)] p-1">
            <button
                v-for="name in TABS"
                :key="name"
                type="button"
                class="flex-1 rounded-[var(--noro-r-sm)] px-3 py-2 text-xs font-bold uppercase tracking-wider transition-colors"
                :class="tab === name
                    ? 'bg-[var(--noro-cream)] text-[var(--noro-on-cream)]'
                    : 'text-[var(--noro-muted)] hover:bg-[var(--noro-panel)]'"
                @click="tab = name"
            >{{ name }}</button>
        </nav>

        <div v-if="loading" class="py-8 text-center text-sm text-[var(--noro-muted)]">
            <UIcon name="i-lucide-loader-circle" class="size-5 animate-spin" />
        </div>

        <template v-else-if="tab === 'versions'">
            <label v-if="hasContext" class="flex items-center gap-2 text-xs text-[var(--noro-muted)]">
                <input
                    type="checkbox"
                    :checked="compatibleOnly"
                    @change="emit('update:compatibleOnly', ($event.target as HTMLInputElement).checked)"
                >
                Only versions matching this pack
            </label>
            <div v-if="loadingVersions" class="py-4 text-center">
                <UIcon name="i-lucide-loader-circle" class="size-4 animate-spin text-[var(--noro-blue)]" />
            </div>
            <div v-else-if="!versions.length" class="py-4 text-center text-sm text-[var(--noro-muted)]">
                No matching versions.
            </div>
            <div v-else class="noro-scroll grid max-h-[480px] gap-2 overflow-y-auto pr-1">
                <ModsVersionRow
                    v-for="version in versions"
                    :key="version.id"
                    :version="version"
                    @install="emit('install', version)"
                />
            </div>
        </template>

        <div v-else-if="tab === 'about'" class="noro-scroll max-h-[480px] overflow-y-auto pr-1">
            <ModsBody :body="project?.body ?? ''" :provider="hit.provider" />
        </div>

        <div v-else class="noro-scroll grid max-h-[480px] gap-3 overflow-y-auto pr-1">
            <img
                v-for="src in project?.gallery ?? []"
                :key="src"
                :src="src"
                alt=""
                loading="lazy"
                class="w-full rounded-[var(--noro-r-sm)]"
            >
            <p v-if="!project?.gallery?.length" class="text-sm text-[var(--noro-muted)]">
                No screenshots.
            </p>
        </div>
    </section>
</template>
