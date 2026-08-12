<script setup lang="ts">
import type { BuildRow } from "~/types/api";

/**
 * Тот же браузер, но в контексте сборки: фильтры совместимости уже стоят по её
 * версии и загрузчику, а сама сборка отмечена целью установки.
 */
const route = useRoute();
const auth = useAuth();
await auth.loadMe();

const serverId = computed(() => String(route.params.id));
const buildId = computed(() => String(route.params.bid));

const { data } = await useAsyncData(`build-mods-${buildId.value}`, () =>
    auth.request<{ build: BuildRow }>(`/api/admin/builds/${buildId.value}`),
);
const build = computed(() => data.value?.build);
</script>

<template>
    <NoroShell
        :title="`MODS · ${build?.version || buildId}`"
        :subtitle="build ? `${build.modloader} ${build.mc_version}` : 'loading'"
    >
        <template #actions>
            <AtomButton
                variant="dark"
                icon="i-lucide-arrow-left"
                :to="`/admin/clients/${serverId}/build/${buildId}`"
            >
                Build
            </AtomButton>
        </template>

        <ModsBrowser
            :server-id="serverId"
            :build-id="buildId"
            :mc="build?.mc_version"
            :loader="build?.modloader"
        />
    </NoroShell>
</template>
