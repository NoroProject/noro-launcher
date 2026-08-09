<script setup lang="ts">
/**
 * Моды на игровом сервере.
 *
 * Список — обычное чтение каталога модов: отдельная операция «дай список
 * модов» была бы тем же `fs_list` с другим именем. Каталог зависит от
 * платформы: у Paper это `plugins`, у лоадеров — `mods`.
 */
const props = defineProps<{
    gameServerId: string;
    serverId: string;
    enabled: boolean;
    platform: string | null;
}>();

const files = useServerFiles(props.gameServerId);
const notify = useNotify();

const PLUGIN_PLATFORMS = ["paper", "bukkit", "spigot", "purpur", "velocity", "bungeecord"];
const dir = computed(() =>
    PLUGIN_PLATFORMS.includes(props.platform ?? "") ? "plugins" : "mods",
);

const jars = computed(() =>
    files.entries.value.filter((e) => !e.dir && e.name.toLowerCase().endsWith(".jar")),
);

const catalogLink = computed(
    () => `/admin/mods?server=${props.serverId}&gs=${props.gameServerId}`,
);

async function remove(path: string, name: string) {
    if (!confirm(`Delete ${name}? The server keeps running it until a restart.`)) return;
    try {
        await files.remove(path);
    } catch (e) {
        notify.fail(e, "Could not delete");
    }
}

watch(
    () => [props.enabled, dir.value],
    ([on]) => {
        if (on) files.open(dir.value);
    },
    { immediate: true },
);
</script>

<template>
    <section class="noro-panel overflow-hidden">
        <div class="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--noro-border)] px-4 py-3">
            <div>
                <span class="noro-label noro-label-inline">{{ dir }}/</span>
                <span class="ml-2 text-xs text-[var(--noro-muted)]">{{ jars.length }} jar(s)</span>
            </div>
            <div class="flex gap-2">
                <AtomButton
                    variant="ghost"
                    size="sm"
                    icon="i-lucide-refresh-cw"
                    :loading="files.loading.value"
                    :disabled="!enabled"
                    aria-label="Refresh"
                    @click="files.open(dir)"
                />
                <AtomButton
                    variant="primary"
                    size="sm"
                    icon="i-lucide-plus"
                    :disabled="!enabled"
                    :to="catalogLink"
                >
                    Add from catalog
                </AtomButton>
            </div>
        </div>

        <UAlert
            v-if="!enabled"
            class="m-4"
            color="warning"
            variant="subtle"
            icon="i-lucide-plug-zap"
            description="No wrapper connected — the mod folder is unreachable."
        />
        <UAlert
            v-else-if="files.error.value"
            class="m-4"
            color="error"
            variant="subtle"
            icon="i-lucide-circle-alert"
            :description="files.error.value"
        />
        <div v-else-if="jars.length" class="divide-y divide-[var(--noro-border)]">
            <div
                v-for="jar in jars"
                :key="jar.path"
                class="flex items-center gap-3 px-4 py-3 transition hover:bg-white/5"
            >
                <UIcon name="i-lucide-package" class="size-4 shrink-0 text-[var(--noro-cream)]" />
                <span class="min-w-0 flex-1 truncate text-sm text-[var(--noro-text)]">{{ jar.name }}</span>
                <span class="shrink-0 text-xs text-[var(--noro-muted)]">{{ compactBytes(jar.size) }}</span>
                <span class="hidden shrink-0 text-xs text-[var(--noro-muted)] sm:block">
                    {{ relativeDate(new Date(jar.modified).toISOString()) }}
                </span>
                <AtomButton
                    variant="ghost"
                    size="sm"
                    icon="i-lucide-trash-2"
                    aria-label="Delete"
                    @click="remove(jar.path, jar.name)"
                />
            </div>
        </div>
        <p v-else class="p-8 text-center text-sm text-[var(--noro-muted)]">
            No jars in {{ dir }}/ yet.
        </p>
    </section>
</template>
