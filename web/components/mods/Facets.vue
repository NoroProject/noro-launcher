<script setup lang="ts">
import type { CatalogCategory, CatalogProviders } from "~/types/catalog";

const props = defineProps<{
    filters: {
        provider: string;
        projectType: string;
        mc: string;
        loader: string;
        categories: string[];
        side: "" | "client" | "server";
    };
    categories: CatalogCategory[];
    providers: CatalogProviders;
    mcVersions: string[];
}>();

defineEmits<{ toggleCategory: [name: string]; reset: [] }>();
const { t } = useT();

const PROJECT_TYPES = ["mod", "modpack", "resourcepack", "shader", "datapack", "plugin"];
const LOADERS = ["", "fabric", "forge", "neoforge", "quilt"];

const providerChips = computed(() => [
    { id: "modrinth", label: "Modrinth", on: props.providers.modrinth },
    { id: "curseforge", label: "CurseForge", on: props.providers.curseforge },
    { id: "all", label: t("admin-facets-both"), on: props.providers.curseforge },
]);

const activeCount = computed(
    () => props.filters.categories.length + (props.filters.side ? 1 : 0),
);
</script>

<template>
    <aside class="noro-panel grid content-start gap-5 p-4">
        <section class="grid gap-2">
            <span class="noro-label noro-label-inline">{{ t('admin-facets-source') }}</span>
            <div class="flex flex-wrap gap-2">
                <button
                    v-for="chip in providerChips"
                    :key="chip.id"
                    type="button"
                    class="noro-chip px-3 py-2 text-xs font-bold uppercase tracking-wider disabled:opacity-40"
                    :class="filters.provider === chip.id ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
                    :disabled="!chip.on"
                    :title="chip.on ? undefined : 'CURSEFORGE_API_KEY is not set'"
                    @click="filters.provider = chip.id"
                >{{ chip.label }}</button>
            </div>
        </section>

        <label class="block">
            <span class="noro-label">{{ t('admin-punish-kind') }}</span>
            <NoroSelect v-model="filters.projectType">
                <option v-for="type in PROJECT_TYPES" :key="type" :value="type">{{ type }}</option>
            </NoroSelect>
        </label>

        <label class="block">
            <span class="noro-label">Minecraft</span>
            <NoroSelect v-model="filters.mc">
                <option value="">{{ t('admin-facets-any-version') }}</option>
                <option v-for="v in mcVersions" :key="v" :value="v">{{ v }}</option>
            </NoroSelect>
        </label>

        <label class="block">
            <span class="noro-label">Loader</span>
            <NoroSelect v-model="filters.loader">
                <option v-for="l in LOADERS" :key="l" :value="l">{{ l || t('admin-facets-any-loader') }}</option>
            </NoroSelect>
        </label>

        <section class="grid gap-2">
            <span class="noro-label noro-label-inline">{{ t('admin-facets-runs-on') }}</span>
            <div class="flex gap-2">
                <button
                    v-for="side in ['', 'client', 'server']"
                    :key="side"
                    type="button"
                    class="noro-chip flex-1 px-2 py-2 text-xs font-bold uppercase tracking-wider"
                    :class="filters.side === side ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
                    @click="filters.side = side as '' | 'client' | 'server'"
                >{{ side || t('admin-facets-any-side') }}</button>
            </div>
        </section>

        <section v-if="categories.length" class="grid gap-2">
            <span class="noro-label noro-label-inline">{{ t('admin-optmods-category') }}</span>
            <div class="noro-scroll grid max-h-72 gap-1 overflow-y-auto pr-1">
                <button
                    v-for="cat in categories"
                    :key="cat.name"
                    type="button"
                    class="flex items-center gap-2 rounded-[var(--noro-r-sm)] px-2 py-2 text-left text-sm transition-colors"
                    :class="filters.categories.includes(cat.name)
                        ? 'bg-[var(--noro-panel-2)] text-[var(--noro-cream)]'
                        : 'text-[var(--noro-muted)] hover:bg-[var(--noro-input)]'"
                    @click="$emit('toggleCategory', cat.name)"
                >
                    <UIcon
                        :name="filters.categories.includes(cat.name) ? 'i-lucide-check-square' : 'i-lucide-square'"
                        class="size-4 shrink-0"
                    />
                    <span class="truncate">{{ cat.display }}</span>
                </button>
            </div>
        </section>

        <AtomButton
            v-if="activeCount"
            variant="ghost"
            size="sm"
            icon="i-lucide-filter-x"
            block
            @click="$emit('reset')"
        >
            {{ t('admin-facets-clear', { count: activeCount }) }}
        </AtomButton>
    </aside>
</template>
