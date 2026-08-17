<script setup lang="ts">
const api = useApi();
const notify = useNotify();
const { t } = useT();
const busy = ref(false);

async function download() {
    busy.value = true;
    try {
        const res = await fetch(`${api.masterUrl.value}/api/admin/backup`, {
            headers: { Authorization: `Bearer ${api.token.value}` },
        });
        if (!res.ok) throw new Error(`Master returned ${res.status}`);

        const name =
            res.headers
                .get("content-disposition")
                ?.match(/filename="([^"]+)"/)?.[1] || "noro.dump";

        const url = URL.createObjectURL(await res.blob());
        const link = document.createElement("a");
        link.href = url;
        link.download = name;
        link.click();
        URL.revokeObjectURL(url);

        notify.ok("Backup downloaded", name);
    } catch (e) {
        notify.fail(e, "Could not create the backup");
    } finally {
        busy.value = false;
    }
}
</script>

<template>
    <div class="grid gap-2">
        <AtomButton
            icon="i-lucide-database-backup"
            variant="secondary"
            :disabled="busy"
            :loading="busy"
            @click="download"
        >
            {{ t('admin-dash-backup-btn') }}
        </AtomButton>
        <p class="text-xs text-[var(--noro-muted)]">
            {{ t('admin-dash-backup-hint') }}
        </p>
    </div>
</template>
