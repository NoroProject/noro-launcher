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
</script>

<template>
    <NoroShell
        :title="server?.name || 'SERVER'"
        subtitle="Server profile control surface"
    >
        <template #actions>
            <AtomButton
                to="/admin/servers"
                icon="i-lucide-arrow-left"
                variant="dark"
            >
                Servers
            </AtomButton>
            <AtomButton
                icon="i-lucide-refresh-cw"
                variant="dark"
                :loading="buildsPending"
                @click="refreshBuilds()"
            >
                Refresh
            </AtomButton>
            <AtomButton
                icon="i-lucide-plus"
                variant="primary"
                @click="showCreateBuild = true"
            >
                New build
            </AtomButton>
        </template>

        <EmptyState
            v-if="!server"
            icon="i-lucide-search-x"
            title="Server not found"
        />
        <div v-else class="grid gap-8">
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
                            <img
                                v-if="server.icon_url"
                                :src="server.icon_url"
                                alt=""
                                class="h-full w-full object-cover"
                            >
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

            <ServerSectionTabs v-model="activeTab" />

            <!-- Без items-start: колонки тянутся до общей высоты, иначе правая
                 панель обрывалась выше левой. Панель ролей вынесена вниз на всю
                 ширину — в колонке она задирала высоту, и справа зияла пустота. -->
            <div class="grid gap-5 xl:grid-cols-[1fr_390px]">
                <ServerSettingsForm
                    :form="form"
                    :active-tab="activeTab"
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

            <ServerGameServersPanel :server-id="id" />

            <ServerBuildsPanel
                :builds="builds"
                :server-id="id"
                :pending="buildsPending"
                :error="buildsError"
                @refresh="refreshBuilds"
                @create="showCreateBuild = true"
            />
        </div>

        <BuildCreateModal
            v-model="showCreateBuild"
            :form="buildForm"
            :minecraft="minecraft"
            :modloaders="modloaders"
            :loader-options="loaderOptions"
            :loading="loading"
            :creating="creatingBuild"
            @create="createBuild"
        />
    </NoroShell>
</template>
