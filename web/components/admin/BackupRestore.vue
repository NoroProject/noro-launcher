<script setup lang="ts">
/**
 * Восстановление мастера из архива.
 *
 * Два шага, а не один: сперва разбор — что за архив, когда собран, сходится ли
 * подпись, — и только потом подтверждение. Восстановление необратимо, и
 * показать это после нажатия было бы поздно.
 *
 * Архив при разборе остаётся на мастере, `restore` берёт его по id: заливать
 * гигабайты дважды ради одного подтверждения незачем.
 */
import type { InspectRes, RestoreRes, Verdict } from '~/types/backup'

const api = useApi()
const notify = useNotify()
const { t } = useT()

const file = ref<File | null>(null)
const uploading = ref(false)
const percent = ref(0)
const upload = ref<string | null>(null)
const verdict = ref<Verdict | null>(null)
const refusal = ref<string | null>(null)
const confirming = ref(false)
const restoring = ref(false)
const done = ref(false)

// Новый файл — старый разбор недействителен: иначе кнопка восстановления
// осталась бы гореть зелёным от предыдущего архива.
watch(file, () => {
    verdict.value = null
    upload.value = null
    refusal.value = null
})

async function inspect() {
    if (!file.value) return
    uploading.value = true
    percent.value = 0
    verdict.value = null
    try {
        const res = await send(file.value)
        upload.value = res.upload
        verdict.value = res.verdict
        refusal.value = res.refusal
    } catch (e) {
        notify.fail(e)
    } finally {
        uploading.value = false
    }
}

/** XHR, а не fetch: прогресса отдачи тела у fetch нет, а архив идёт минутами. */
function send(f: File) {
    return new Promise<InspectRes>((resolve, reject) => {
        const xhr = new XMLHttpRequest()
        xhr.open('POST', `${api.masterUrl.value}/api/admin/backup/inspect`)
        xhr.setRequestHeader('Authorization', `Bearer ${api.token.value}`)
        xhr.upload.onprogress = (e) => {
            if (e.lengthComputable) percent.value = Math.round((e.loaded / e.total) * 100)
        }
        xhr.onload = () => {
            const body = JSON.parse(xhr.responseText || '{}')
            if (xhr.status < 400) resolve(body)
            else reject(new Error(body.error || `HTTP ${xhr.status}`))
        }
        xhr.onerror = () => reject(new Error(t('admin-backup-upload-failed')))
        xhr.send(f)
    })
}

async function restore() {
    restoring.value = true
    try {
        const out = await api.request<RestoreRes>('/api/admin/backup/restore', {
            method: 'POST',
            body: { upload: upload.value },
        })
        confirming.value = false
        done.value = true
        notify.ok(t('admin-backup-restore-done'), out.previous_data)
    } catch (e) {
        notify.fail(e)
    } finally {
        restoring.value = false
    }
}
</script>

<template>
    <section class="noro-panel grid gap-3 p-4">
        <span class="noro-label">{{ t('admin-backup-restore') }}</span>
        <p class="text-sm leading-6 text-[var(--noro-text)]">
            {{ t('admin-backup-restore-text') }}
        </p>

        <AdminBackupDropzone v-model="file" :disabled="uploading || restoring" />

        <div class="flex flex-wrap items-center gap-4">
            <AtomButton
                icon="i-lucide-search-check"
                :disabled="!file"
                :loading="uploading"
                @click="inspect"
            >
                {{ t('admin-backup-inspect-btn') }}
            </AtomButton>
            <span v-if="uploading" class="text-xs text-[var(--noro-muted)]">
                {{ t('admin-backup-uploading', { percent }) }}
            </span>
        </div>

        <template v-if="verdict">
            <AdminBackupVerdict :verdict="verdict" :refusal="refusal" />
            <NoroNote icon="i-lucide-alert-triangle">{{ t('admin-backup-swap-note') }}</NoroNote>
            <AtomButton
                icon="i-lucide-history"
                variant="danger"
                :disabled="!!refusal || done"
                @click="confirming = true"
            >
                {{ t('admin-backup-restore-btn') }}
            </AtomButton>
        </template>

        <NoroModal v-model="confirming" :title="t('admin-backup-confirm-title')">
            <div class="grid gap-4">
                <p class="text-sm leading-6 text-[var(--noro-text)]">
                    {{ t('admin-backup-confirm-text') }}
                </p>
                <AdminBackupVerdict v-if="verdict" :verdict="verdict" :refusal="refusal" />
                <AtomButton variant="danger" :loading="restoring" block @click="restore">
                    {{ t('admin-backup-confirm-btn') }}
                </AtomButton>
            </div>
        </NoroModal>
    </section>
</template>
