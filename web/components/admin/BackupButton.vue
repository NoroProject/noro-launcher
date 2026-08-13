<script setup lang="ts">
/**
 * Скачивание дампа БД мастера.
 *
 * Обычной ссылкой не обойтись: эндпоинт требует Bearer-токен, а `<a href>` его
 * не отправит. Поэтому запрос идёт через fetch, а файл отдаётся браузеру
 * временной blob-ссылкой.
 */
const api = useApi();
const notify = useNotify();
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
            Download database backup
        </AtomButton>
        <p class="text-xs text-[var(--noro-muted)]">
            A pg_restore archive of the master database — accounts, permissions,
            skins and capes. Keep it off this machine.
        </p>
    </div>
</template>
