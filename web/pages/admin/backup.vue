<script setup lang="ts">
/**
 * Резервная копия мастера.
 *
 * Три вещи в одном месте: дамп одной базы, полный архив вместе с файлами и
 * восстановление из архива. Дамп оставлен отдельно как быстрый способ забрать
 * базу, когда файлы копируются иначе.
 */
const auth = useAuth()
const api = useApi()
const { t } = useT()
const notify = useNotify()
await auth.loadMe()

const busy = ref(false)

async function download() {
    busy.value = true
    try {
        // Обычной ссылкой не обойтись: ручка требует Bearer-токен, а <a download>
        // заголовки не носит. Поэтому качаем сами и отдаём файл через Blob.
        const res = await fetch(`${api.masterUrl.value}/api/admin/backup`, {
            headers: { Authorization: `Bearer ${api.token.value}` },
        })
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        const blob = await res.blob()
        const url = URL.createObjectURL(blob)
        const link = document.createElement('a')
        link.href = url
        link.download = `noro-${new Date().toISOString().slice(0, 10)}.dump`
        link.click()
        URL.revokeObjectURL(url)
        notify.ok()
    } catch (e) {
        notify.fail(e)
    } finally {
        busy.value = false
    }
}
</script>

<template>
    <NoroShell :title="t('admin-backup-title')" :subtitle="t('admin-backup-subtitle')">
        <div class="grid gap-4">
            <!-- Две выгрузки рядом: выбор между ними — это одно решение, и
                 разносить их по вертикали значит заставлять сравнивать по
                 памяти. Восстановление ниже и во всю ширину: у него разбор
                 архива таблицей. -->
            <div class="grid gap-4 lg:grid-cols-2">
                <section class="noro-panel flex flex-col gap-3 p-4">
                    <span class="noro-label">{{ t('admin-backup-db') }}</span>
                    <p class="flex-1 text-sm leading-6 text-[var(--noro-text)]">
                        {{ t('admin-backup-db-text') }}
                    </p>
                    <AtomButton
                        icon="i-lucide-download"
                        variant="secondary"
                        :loading="busy"
                        @click="download"
                    >
                        {{ t('admin-backup-download') }}
                    </AtomButton>
                </section>

                <AdminBackupArchive />
            </div>

            <AdminBackupRestore v-if="auth.hasPermission('noro.admin.backup.restore')" />

            <NoroNote icon="i-lucide-info">{{ t('admin-backup-files-note') }}</NoroNote>
        </div>
    </NoroShell>
</template>
