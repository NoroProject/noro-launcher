<script setup lang="ts">
/** Управление бэкапами серверной директории. */
const props = defineProps<{
    gameServerId: string;
    enabled: boolean;
}>();

const backups = useServerBackups(props.gameServerId);
const label = ref("");

onMounted(() => {
    if (props.enabled) {
        backups.refresh();
    }
});

watch(() => props.enabled, (on) => {
    if (on) {
        backups.refresh();
    }
});

function handleCreate() {
    backups.create(label.value.trim());
    label.value = "";
}

function formatSize(bytes: number) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(ts: number) {
    return new Date(ts).toLocaleString();
}
</script>

<template>
    <div class="noro-panel p-5 grid gap-4">
        <div class="flex items-center justify-between gap-4">
            <div>
                <h3 class="text-sm font-bold uppercase tracking-wider text-[var(--noro-text)]">
                    Server Snapshots
                </h3>
                <p class="text-xs text-[var(--noro-muted)]">
                    Quick zip backups of server configs and worlds before updates.
                </p>
            </div>
            <AtomButton
                variant="dark"
                icon="i-lucide-refresh-cw"
                :disabled="!enabled || backups.loading.value"
                @click="backups.refresh()"
            >
                Refresh
            </AtomButton>
        </div>

        <form class="flex items-center gap-3" @submit.prevent="handleCreate">
            <input
                v-model="label"
                type="text"
                placeholder="Optional snapshot note (e.g. pre-mod-update)"
                class="noro-input text-xs flex-1"
                :disabled="!enabled || backups.loading.value"
            />
            <AtomButton
                variant="accent"
                icon="i-lucide-archive"
                type="submit"
                :disabled="!enabled || backups.loading.value"
            >
                Create Snapshot
            </AtomButton>
        </form>

        <div v-if="!enabled" class="text-center py-8 text-xs text-[var(--noro-muted)]">
            ServerWrapper is offline. Connect wrapper to manage backups.
        </div>

        <div v-else-if="backups.loading.value && !backups.backups.value.length" class="text-center py-8 text-xs text-[var(--noro-muted)]">
            Loading snapshots...
        </div>

        <div v-else-if="backups.error.value" class="text-xs text-red-400 p-3 bg-red-950/20 border border-red-900/40 rounded-lg">
            {{ backups.error.value }}
        </div>

        <div v-else-if="!backups.backups.value.length" class="text-center py-8 text-xs text-[var(--noro-muted)]">
            No snapshots found.
        </div>

        <div v-else class="grid gap-2">
            <div
                v-for="b in backups.backups.value"
                :key="b.name"
                class="flex items-center justify-between gap-4 p-3 bg-[var(--noro-bg-subtle)] rounded-lg border border-[var(--noro-border)] text-xs"
            >
                <div class="grid gap-0.5 min-w-0">
                    <span class="font-mono font-bold text-[var(--noro-text)] truncate">{{ b.name }}</span>
                    <span class="text-[10px] text-[var(--noro-muted)]">
                        {{ formatDate(b.created) }} • {{ formatSize(b.size) }}
                    </span>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                    <AtomButton
                        variant="dark"
                        icon="i-lucide-rotate-ccw"
                        :disabled="backups.loading.value"
                        @click="backups.restore(b.name)"
                    >
                        Restore
                    </AtomButton>
                    <AtomButton
                        variant="danger"
                        icon="i-lucide-trash-2"
                        :disabled="backups.loading.value"
                        @click="backups.remove(b.name)"
                    >
                        Delete
                    </AtomButton>
                </div>
            </div>
        </div>
    </div>
</template>
