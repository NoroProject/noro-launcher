<script setup lang="ts">
import type { InstalledModInfo, ModHit, ModProject, ModVersion } from "~/types/catalog";

/** Панель проекта: версии, описание, галерея. Правая колонка браузера. */
defineProps<{
    hit: ModHit;
    project: ModProject | null;
    versions: ModVersion[];
    loading?: boolean;
    loadingVersions?: boolean;
    compatibleOnly: boolean;
    hasContext: boolean;
    installedMod?: InstalledModInfo | null;
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
    <section class="noro-panel flex flex-col h-full max-h-[calc(100vh-7rem)] p-4 gap-4 overflow-hidden">
        <header class="flex items-start gap-3 shrink-0">
            <img
                v-if="hit.icon_url"
                :src="hit.icon_url"
                alt=""
                class="size-11 shrink-0 rounded-[var(--noro-r-sm)] object-cover bg-[var(--noro-bg-deep)]"
            >
            <div class="min-w-0 flex-1">
                <h2 class="truncate font-bold text-[var(--noro-cream)] text-base" :title="hit.title">{{ hit.title }}</h2>
                <p class="mt-0.5 truncate text-xs text-[var(--noro-muted)]">
                    {{ hit.author }} · {{ compactNumber(hit.downloads) }} downloads
                </p>
            </div>
            <AtomButton variant="ghost" size="sm" icon="i-lucide-x" aria-label="Close" class="shrink-0" @click="emit('close')" />
        </header>

        <div v-if="hit.page_url || project?.source_url || project?.issues_url" class="flex flex-wrap gap-2 shrink-0">
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

        <nav class="flex gap-1 rounded-[var(--noro-r-sm)] bg-[var(--noro-bg-deep)] p-1 shrink-0">
            <button
                v-for="name in TABS"
                :key="name"
                type="button"
                class="flex-1 rounded-[var(--noro-r-sm)] px-2.5 py-1.5 text-xs font-bold uppercase tracking-wider transition-colors"
                :class="tab === name
                    ? 'bg-[var(--noro-cream)] text-[var(--noro-on-cream)] shadow-sm'
                    : 'text-[var(--noro-muted)] hover:bg-[var(--noro-panel)] hover:text-[var(--noro-text)]'"
                @click="tab = name"
            >{{ name }}</button>
        </nav>

        <div v-if="loading" class="py-8 text-center text-sm text-[var(--noro-muted)] shrink-0">
            <UIcon name="i-lucide-loader-circle" class="size-5 animate-spin" />
        </div>

        <div v-else class="flex-1 min-h-0 flex flex-col gap-3">
            <template v-if="tab === 'versions'">
                <label v-if="hasContext" class="flex items-center gap-2 text-xs text-[var(--noro-muted)] shrink-0 cursor-pointer select-none">
                    <input
                        type="checkbox"
                        :checked="compatibleOnly"
                        class="rounded accent-[var(--noro-cream)]"
                        @change="emit('update:compatibleOnly', ($event.target as HTMLInputElement).checked)"
                    >
                    <span>Only versions matching this pack</span>
                </label>
                <div v-if="loadingVersions" class="py-8 text-center">
                    <UIcon name="i-lucide-loader-circle" class="size-5 animate-spin text-[var(--noro-blue)]" />
                </div>
                <div v-else-if="!versions.length" class="py-8 text-center text-sm text-[var(--noro-muted)]">
                    No matching versions.
                </div>
                <div v-else class="noro-scroll flex-1 min-h-0 flex flex-col gap-2 overflow-y-auto overflow-x-hidden pr-1.5 w-full">
                    <ModsVersionRow
                        v-for="version in versions"
                        :key="version.id"
                        :version="version"
                        :installed-mod="installedMod"
                        @install="emit('install', version)"
                    />
                </div>
            </template>

            <div v-else-if="tab === 'about'" class="noro-scroll flex-1 min-h-0 overflow-y-auto pr-1.5">
                <ModsBody :body="project?.body ?? ''" :provider="hit.provider" />
            </div>

            <div v-else class="noro-scroll flex-1 min-h-0 grid content-start gap-3 overflow-y-auto pr-1.5">
                <img
                    v-for="src in project?.gallery ?? []"
                    :key="src"
                    :src="src"
                    alt=""
                    loading="lazy"
                    class="w-full rounded-[var(--noro-r-sm)]"
                >
                <p v-if="!project?.gallery?.length" class="text-sm text-[var(--noro-muted)] py-4 text-center">
                    No screenshots.
                </p>
            </div>
        </div>
    </section>
</template>
