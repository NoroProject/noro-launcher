<script setup lang="ts">
/**
 * Полный архив мастера: дамп базы, файлы данных, паспорт и подпись.
 *
 * Скачивает сам браузер, обычным переходом по ссылке. Раньше здесь был `fetch`
 * с ручной сборкой кусков — заголовок `Authorization` иначе не поставить, — и
 * весь архив копился в памяти вкладки: на паре гигабайт это минуты ожидания на
 * ровном месте. `showSaveFilePicker` спас бы, но его нет в Safari.
 *
 * Поэтому мастер выдаёт одноразовый билет, а дальше работает браузер: поток
 * прямо на диск, свой индикатор загрузки и докачка при обрыве.
 */
const api = useApi()
const notify = useNotify()
const { t } = useT()

const busy = ref(false)

async function download() {
    busy.value = true
    try {
        const { ticket } = await api.request<{ ticket: string }>('/api/admin/backup/ticket', {
            method: 'POST',
        })
        // Не `window.open`: всплывающее окно режут блокировщики, а скачивание
        // по прямому переходу браузер обрабатывает как загрузку и страницу не
        // покидает — заголовок Content-Disposition ему это и говорит.
        window.location.href = `${api.masterUrl.value}/api/admin/backup/full/${ticket}`
        notify.info(t('admin-backup-full-started'))
    } catch (e) {
        notify.fail(e)
    } finally {
        busy.value = false
    }
}
</script>

<template>
    <section class="noro-panel flex flex-col gap-3 p-4">
        <span class="noro-label">{{ t('admin-backup-full') }}</span>
        <p class="flex-1 text-sm leading-6 text-[var(--noro-text)]">
            {{ t('admin-backup-full-text') }}
        </p>

        <AtomButton icon="i-lucide-archive" variant="primary" :loading="busy" @click="download">
            {{ t('admin-backup-full-btn') }}
        </AtomButton>
    </section>
</template>
