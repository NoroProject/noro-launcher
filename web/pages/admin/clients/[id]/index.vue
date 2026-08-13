<script setup lang="ts">
const {
    id,
    server,
    form,
    activeTab,
    minecraft,
    modloaders,
    loading,
    save,
    remove,
    saving,
    deleting,
    buildsData,
    buildForm,
    loaderOptions,
    creatingBuild,
    showCreateBuild,
    createBuild,
    roles,
    togglingRole,
    hasAccess,
    toggleRoleAccess,
    applyAsset,
} = await useAdminServerEditor();

const builds = computed(() => buildsData.data.value || []);
const buildsPending = buildsData.pending;
const buildsError = buildsData.error;
const refreshBuilds = buildsData.refresh;

const selectedBuildId = ref<string | null>(null);

watch(
    builds,
    (list) => {
        if (!list.length) {
            selectedBuildId.value = null;
            return;
        }
        if (!selectedBuildId.value || !list.some((b) => b.id === selectedBuildId.value)) {
            const pub = list.find((b) => b.published);
            selectedBuildId.value = pub ? pub.id : list[0].id;
        }
    },
    { immediate: true },
);

const buildEditor = useAdminBuildEditor(
    id,
    selectedBuildId,
    () => {
        refreshBuilds();
        selectedBuildId.value = null;
    },
);
</script>

<template>
    <NoroShell
        :title="server?.name || 'SERVER'"
        subtitle="Server profile & assembly control surface"
    >
        <template #actions>
            <AtomButton to="/admin/clients" icon="i-lucide-arrow-left" variant="dark">
                Clients
            </AtomButton>
            <AtomButton
                icon="i-lucide-refresh-cw"
                variant="dark"
                :loading="buildsPending"
                @click="refreshBuilds()"
            >
                Refresh
            </AtomButton>
        </template>

        <EmptyState v-if="!server" icon="i-lucide-search-x" title="Server not found" />
        <div v-else class="grid gap-6">
            <!-- Hero Panel -->
            <section class="noro-hero-panel overflow-hidden">
                <img
                    v-if="server.background_url"
                    :src="server.background_url"
                    alt=""
                    class="absolute inset-0 h-full w-full object-cover opacity-35"
                >
                <div class="relative z-10 flex flex-wrap items-end justify-between gap-6 p-6">
                    <div class="flex min-w-0 items-center gap-4">
                        <div class="grid size-16 place-items-center overflow-hidden rounded border border-[var(--noro-border)] bg-[var(--noro-input)]">
                            <img v-if="server.icon_url" :src="server.icon_url" alt="" class="h-full w-full object-cover">
                            <UIcon v-else name="i-lucide-server" class="size-7 text-[var(--noro-blue)]" />
                        </div>
                        <div class="min-w-0">
                            <h2 class="noro-pixel truncate text-xl text-[var(--noro-cream)]">
                                {{ server.name }}
                            </h2>
                            <p class="mt-2 text-sm text-[var(--noro-muted)]">
                                {{ form.description || "No description" }}
                            </p>
                        </div>
                    </div>
                    <div class="flex flex-wrap gap-2">
                        <UBadge :color="form.active ? 'success' : 'neutral'" variant="subtle">
                            {{ form.active ? "active" : "offline" }}
                        </UBadge>
                        <UBadge v-if="form.limited" color="warning" variant="subtle">
                            limited
                        </UBadge>
                        <UBadge color="neutral" variant="subtle">
                            {{ form.modloader }} {{ form.mc_version }}
                        </UBadge>
                    </div>
                </div>
            </section>

            <!-- Navigation Tabs -->
            <ServerSectionTabs v-model="activeTab" />

            <!-- Build Selector Bar (shown on Mods, Build & Client tabs) -->
            <ServerBuildSelectorBar
                v-if="activeTab === 'mods' || activeTab === 'build' || activeTab === 'client'"
                :builds="builds"
                :selected-id="selectedBuildId"
                :pending="buildsPending"
                @select="selectedBuildId = $event"
                @create="showCreateBuild = true"
            />

            <!-- TAB 1: Profile & Assets -->
            <template v-if="activeTab === 'server'">
                <div class="grid gap-5 xl:grid-cols-[1fr_390px]">
                    <ServerSettingsForm
                        :form="form"
                        active-tab="server"
                        :minecraft="minecraft"
                        :modloaders="modloaders"
                        :loading-minecraft="loading"
                        :saving="saving"
                        :deleting="deleting"
                        @save="save"
                        @remove="remove"
                    />
                    <ServerMediaPanel
                        :server-id="id"
                        :icon-url="server.icon_url"
                        :background-url="server.background_url"
                        @uploaded="applyAsset"
                    />
                </div>

                <ServerRolesPanel
                    v-if="form.limited"
                    :roles="roles"
                    :toggling-role="togglingRole"
                    :has-access="hasAccess"
                    @toggle="toggleRoleAccess"
                />
            </template>

            <!-- TAB 2: Installed Mods & Catalog -->
            <template v-else-if="activeTab === 'mods'">
                <ServerInstalledModsPanel
                    :server-id="id"
                    :build-id="selectedBuildId"
                    :files="buildEditor.filesData.data.value || []"
                    :busy="buildEditor.busy.value"
                    @remove="buildEditor.removeFile"
                    @upload="(file) => { buildEditor.fileUpload.value = file; buildEditor.uploadBuildFile(); }"
                />
            </template>

            <!-- TAB 2: Build & Files -->
            <template v-else-if="activeTab === 'build'">
                <EmptyState
                    v-if="!selectedBuildId"
                    icon="i-lucide-box"
                    title="No builds created yet"
                    text="Create a build version to manage files, import modpacks, and release to clients."
                />
                <div v-else class="grid gap-5">
                    <UAlert
                        v-if="buildEditor.message.value"
                        :color="buildEditor.messageError.value ? 'error' : 'success'"
                        variant="subtle"
                        :icon="buildEditor.messageError.value ? 'i-lucide-circle-alert' : 'i-lucide-check'"
                        :description="buildEditor.message.value"
                    />

                    <!-- Import progress notification -->
                    <div v-if="buildEditor.importProgress.value && !buildEditor.importProgress.value.done" class="noro-panel p-5 bg-noro-blue/10 border-noro-blue/30">
                        <h3 class="text-sm font-bold text-noro-blue mb-2 flex items-center gap-2">
                            <UIcon name="i-lucide-loader-2" class="animate-spin" />
                            Importing Modpack...
                        </h3>
                        <div class="mb-1 text-xs text-noro-muted flex justify-between">
                            <span class="truncate pr-4">{{ buildEditor.importProgress.value.current_file }}</span>
                            <span class="shrink-0">{{ buildEditor.importProgress.value.current }} / {{ buildEditor.importProgress.value.total }}</span>
                        </div>
                        <UProgress :value="buildEditor.importProgress.value.current" :max="buildEditor.importProgress.value.total || 1" color="info" size="sm" />
                    </div>

                    <div class="grid gap-5 xl:grid-cols-[1fr_390px] items-start">
                        <div class="grid gap-4 content-start">
                            <BuildPublishPanel
                                :build="buildEditor.build.value"
                                :busy="buildEditor.busy.value"
                                :build-pending="buildEditor.buildPayload.pending.value"
                                @publish="buildEditor.publish"
                                @rebuild="buildEditor.rebuild"
                                @rebuild-clean="buildEditor.rebuildClean"
                                @unpublish="buildEditor.unpublish"
                                @delete="buildEditor.deleteBuild"
                            />
                            <BuildFilesSummary
                                :files="buildEditor.filesData.data.value"
                                :build-id="selectedBuildId"
                                @open-manager="buildEditor.showFileManager.value = true"
                            />
                        </div>

                        <aside class="grid gap-5 content-start">
                            <BuildImportPanel
                                v-model:import-file="buildEditor.importFile.value"
                                v-model:import-kind="buildEditor.importKind.value"
                                :busy="buildEditor.busy.value"
                                @submit-import="buildEditor.importPack"
                            />
                            <BuildManualFilePanel
                                v-model:file-upload="buildEditor.fileUpload.value"
                                v-model:file-path="buildEditor.filePath.value"
                                :busy="buildEditor.busy.value"
                                @upload="buildEditor.uploadBuildFile"
                            />
                        </aside>
                    </div>

                    <BuildFileManagerModal
                        v-model="buildEditor.showFileManager.value"
                        :build-id="selectedBuildId"
                        @changed="buildEditor.filesData.refresh()"
                    />
                </div>
            </template>

            <!-- TAB 3: Game Servers -->
            <template v-else-if="activeTab === 'instances'">
                <ServerGameServersPanel :server-id="id" />
            </template>

            <!-- TAB 4: Client Defaults & Settings -->
            <template v-else-if="activeTab === 'client'">
                <div class="grid gap-5 xl:grid-cols-[1fr_390px]">
                    <div class="grid gap-5 content-start">
                        <ServerSettingsForm
                            :form="form"
                            active-tab="client"
                            :minecraft="minecraft"
                            :modloaders="modloaders"
                            :loading-minecraft="loading"
                            :saving="saving"
                            :deleting="deleting"
                            @save="save"
                            @remove="remove"
                        />

                        <BuildOptionalModsPanel
                            v-if="selectedBuildId"
                            :optional-mods="buildEditor.optionalData.data.value"
                            :allow-suggestions="buildEditor.build.value?.allow_optional_mod_suggestions ?? true"
                            :busy="buildEditor.busy.value"
                            @save="buildEditor.saveOptional"
                            @delete="buildEditor.deleteOptional"
                            @update:allow-suggestions="buildEditor.toggleAllowSuggestions"
                        />
                    </div>

                    <div v-if="selectedBuildId" class="grid gap-4 content-start">
                        <BuildRecommendedSettingsPanel
                            :form="buildEditor.recommendedForm"
                            :busy="buildEditor.busy.value"
                            @save="buildEditor.saveRecommended"
                        />
                        <BuildPathsPanel
                            :paths-form="buildEditor.pathsForm"
                            @browse="buildEditor.showFileManager.value = true"
                        />
                    </div>
                </div>
            </template>
        </div>

        <BuildCreateModal
            v-model="showCreateBuild"
            :form="buildForm"
            :builds="builds"
            :minecraft="minecraft"
            :modloaders="modloaders"
            :loader-options="loaderOptions"
            :loading="loading"
            :creating="creatingBuild"
            @create="createBuild"
        />
    </NoroShell>
</template>
