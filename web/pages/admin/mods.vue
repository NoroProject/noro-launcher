<script setup lang="ts">
/**
 * Каталог модов без привязки к сборке: искать можно всё, а куда ставить —
 * выбирается в момент установки.
 */
const auth = useAuth();
const { t } = useT();
await auth.loadMe();

// Контекст приходит ссылкой со страницы сборки или игрового сервера: цель
// установки тогда отмечена заранее, и лишний клик не нужен.
const route = useRoute();
const serverId = computed(() => (route.query.server as string) || undefined);
const buildId = computed(() => (route.query.build as string) || undefined);
const gameServerId = computed(() => (route.query.gs as string) || undefined);
</script>

<template>
    <NoroShell :title="t('admin-mods-title')" :subtitle="t('admin-mods-subtitle')">
        <ModsBrowser
            :server-id="serverId"
            :build-id="buildId"
            :game-server-id="gameServerId"
        />
    </NoroShell>
</template>
