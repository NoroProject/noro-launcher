<script setup lang="ts">
import type { InstalledModInfo, ModHit, ModSource, ModVersion } from "~/types/catalog";

/**
 * Браузер каталога модов: фасеты, выдача, карточка проекта.
 *
 * Один компонент на два входа — глобальный `/admin/mods` и страницу сборки.
 * Разница только в контексте: он подставляет версию игры с загрузчиком в
 * фильтры и заранее отмечает сборку целью установки.
 */
const props = defineProps<{
    serverId?: string;
    buildId?: string;
    /** Пришли со страницы игрового сервера — он и будет целью по умолчанию. */
    gameServerId?: string;
    mc?: string;
    loader?: string;
}>();

const auth = useAuth();
const notify = useNotify();
const versions = useVersionOptions();
const catalog = useModCatalog(() => ({ mc: props.mc, loader: props.loader }));
const detail = useModProject();

const pickerOpen = ref(false);
const pending = ref<{ source: ModSource; name: string; hit: ModHit } | null>(null);
const installedMods = ref<InstalledModInfo[]>([]);

async function loadInstalledMods() {
    let targetBuildId = props.buildId;
    if (!targetBuildId && props.serverId) {
        try {
            const builds = await auth.request<Array<{ id: string }>>(`/api/admin/builds?server_id=${props.serverId}`);
            if (builds?.length) {
                targetBuildId = builds[0].id;
            }
        } catch {}
    }
    if (targetBuildId) {
        try {
            const res = await auth.request<InstalledModInfo[]>(`/api/admin/builds/${targetBuildId}/installed-mods`);
            if (res) installedMods.value = res;
        } catch {}
    }
}

function findInstalledMod(hitTitle: string, projectId?: string): InstalledModInfo | null {
    const cleanTitle = hitTitle.replace(/[^a-zA-Z0-9]/g, "").toLowerCase();
    const cleanProj = (projectId || "").replace(/[^a-zA-Z0-9]/g, "").toLowerCase();
    if (!cleanTitle && !cleanProj) return null;

    const found = installedMods.value.find((m) => {
        const cleanName = (m.name || "").replace(/[^a-zA-Z0-9]/g, "").toLowerCase();
        const cleanModId = (m.mod_id || "").replace(/[^a-zA-Z0-9]/g, "").toLowerCase();
        const cleanPath = (m.path || "").replace(/[^a-zA-Z0-9]/g, "").toLowerCase();

        return (
            (cleanName && (cleanTitle.includes(cleanName) || cleanName.includes(cleanTitle))) ||
            (cleanModId && (cleanTitle.includes(cleanModId) || cleanModId.includes(cleanProj))) ||
            (cleanPath && cleanPath.includes(cleanTitle))
        );
    });
    return found ?? null;
}

function sourceFor(version: ModVersion): ModSource {
    return version.provider === "modrinth"
        ? { kind: "modrinth", version_id: version.id }
        : {
              kind: "curseforge",
              project_id: Number(version.project_id),
              file_id: Number(version.id),
          };
}

function startInstall(hit: ModHit, version: ModVersion) {
    pending.value = { source: sourceFor(version), name: hit.title, hit };
    pickerOpen.value = true;
}

/** Установка прямо из карточки: берём последнюю совместимую версию сами. */
async function quickInstall(hit: ModHit) {
    const version = await detail.fetchLatest(hit, catalog.filters.mc, catalog.filters.loader);
    if (!version) {
        notify.fail(new Error("No installable version matches these filters"), hit.title);
        await detail.open(hit, catalog.filters.mc, catalog.filters.loader);
        return;
    }
    startInstall(hit, version);
}

const hasContext = computed(() => Boolean(catalog.filters.mc || catalog.filters.loader));

onMounted(async () => {
    await Promise.all([
        catalog.loadProviders(),
        versions.loadMinecraft().catch(() => {}),
        loadInstalledMods(),
    ]);

    if (!catalog.filters.mc || !catalog.filters.loader) {
        if (props.buildId) {
            try {
                const res = await auth.request<{ build: { version: string; mc_version: string; modloader: string } }>(
                    `/api/admin/builds/${props.buildId}`,
                );
                const mcVersion = res?.build?.mc_version || res?.build?.version;
                if (mcVersion && !catalog.filters.mc) {
                    catalog.filters.mc = mcVersion;
                }
                if (res?.build?.modloader && !catalog.filters.loader) {
                    catalog.filters.loader = res.build.modloader;
                }
            } catch {
                // Ignore build fetch error
            }
        } else if (props.serverId) {
            try {
                const builds = await auth.request<Array<{ id: string; version: string; mc_version: string; modloader: string }>>(
                    `/api/admin/builds?server_id=${props.serverId}`,
                );
                if (builds?.length) {
                    const latest = builds[0];
                    const mcVersion = latest.mc_version || latest.version;
                    if (mcVersion && !catalog.filters.mc) {
                        catalog.filters.mc = mcVersion;
                    }
                    if (latest.modloader && !catalog.filters.loader) {
                        catalog.filters.loader = latest.modloader;
                    }
                }
            } catch {
                // Ignore server builds fetch error
            }
        }
    }

    await Promise.all([catalog.loadCategories(), catalog.search()]);
});
</script>

<template>
    <div
        class="grid items-start gap-5"
        :class="detail.hit.value ? 'xl:grid-cols-[248px_1fr_420px]' : 'xl:grid-cols-[248px_1fr]'"
    >
        <ModsFacets
            :filters="catalog.filters"
            :categories="catalog.categories.value"
            :providers="catalog.providers.value"
            :mc-versions="versions.minecraft.value"
            class="xl:sticky xl:top-24 xl:max-h-[calc(100vh-7rem)] xl:overflow-y-auto noro-scroll"
            @toggle-category="catalog.toggleCategory"
            @reset="catalog.resetFilters"
        />

        <div class="grid content-start gap-4 min-w-0">
            <ModsSearchBar
                v-model="catalog.filters.q"
                v-model:sort="catalog.filters.sort"
                :total="catalog.total.value"
                :loading="catalog.loading.value"
            />

            <UAlert
                v-if="catalog.failed.value.length"
                color="warning"
                variant="subtle"
                icon="i-lucide-triangle-alert"
                :description="`No answer from: ${catalog.failed.value.join(', ')}`"
            />
            <UAlert
                v-if="catalog.error.value"
                color="error"
                variant="subtle"
                icon="i-lucide-circle-alert"
                :description="catalog.error.value"
            />

            <div v-if="catalog.hits.value.length" class="noro-scroll flex flex-col gap-3 max-h-[calc(100vh-16rem)] overflow-y-auto overflow-x-hidden pr-1.5 w-full">
                <ModsHitCard
                    v-for="hit in catalog.hits.value"
                    :key="`${hit.provider}:${hit.project_id}`"
                    :hit="hit"
                    :selected="detail.hit.value?.project_id === hit.project_id"
                    :installed-mod="findInstalledMod(hit.title, hit.project_id)"
                    @select="detail.open(hit, catalog.filters.mc, catalog.filters.loader)"
                    @install="quickInstall(hit)"
                />
            </div>
            <EmptyState
                v-else-if="!catalog.loading.value"
                icon="i-lucide-package-search"
                title="Nothing found"
                text="Loosen the filters or try another term."
            />

            <nav v-if="catalog.pageCount.value > 1" class="flex items-center justify-center gap-4">
                <AtomButton
                    variant="secondary"
                    size="sm"
                    icon="i-lucide-chevron-left"
                    :disabled="catalog.page.value === 0"
                    @click="catalog.goToPage(catalog.page.value - 1)"
                />
                <span class="text-xs text-[var(--noro-muted)]">
                    {{ catalog.page.value + 1 }} / {{ catalog.pageCount.value }}
                </span>
                <AtomButton
                    variant="secondary"
                    size="sm"
                    icon="i-lucide-chevron-right"
                    :disabled="catalog.page.value + 1 >= catalog.pageCount.value"
                    @click="catalog.goToPage(catalog.page.value + 1)"
                />
            </nav>
        </div>

        <ModsProjectPanel
            v-if="detail.hit.value"
            v-model:compatible-only="detail.compatibleOnly.value"
            :hit="detail.hit.value"
            :project="detail.project.value"
            :versions="detail.versions.value"
            :loading="detail.loading.value"
            :loading-versions="detail.loadingVersions.value"
            :has-context="hasContext"
            :installed-mod="findInstalledMod(detail.hit.value.title, detail.hit.value.project_id)"
            class="xl:sticky xl:top-24 xl:h-[calc(100vh-7rem)]"
            @close="detail.close"
            @install="version => startInstall(detail.hit.value!, version)"
        />

        <ModsTargetPicker
            v-model="pickerOpen"
            :source="pending?.source ?? null"
            :mod-name="pending?.name ?? ''"
            :icon-url="pending?.hit.icon_url"
            :author="pending?.hit.author"
            :server-id="serverId"
            :build-id="buildId"
            :game-server-id="gameServerId"
            @installed="loadInstalledMods"
        />
    </div>
</template>
