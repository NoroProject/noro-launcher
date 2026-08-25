<script setup lang="ts">
/**
 * Разбор архива перед восстановлением.
 *
 * Показывается до подтверждения и намеренно подробно: восстановление
 * необратимо, и «дата, схема, подпись» — ровно те три вещи, по которым видно,
 * тот ли это архив.
 */
import type { Verdict } from '~/types/backup'

const props = defineProps<{ verdict: Verdict; refusal: string | null }>()
const { t } = useT()

const meta = computed(() => props.verdict.meta)

const total = computed(() => {
    const mb = (meta.value.dump_bytes + meta.value.data_bytes) / 1_048_576
    return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${mb.toFixed(1)} MB`
})

const schema = computed(() =>
    t('admin-backup-schema-pair', {
        archive: meta.value.schema_version,
        master: props.verdict.known_schema_version,
    }),
)

/** Неподписанный архив — не то же самое, что архив с битой подписью. */
const signature = computed(() => {
    if (!meta.value.signed) return { text: t('admin-backup-unsigned'), ok: false, warn: true }
    return props.verdict.signature_ok
        ? { text: t('admin-backup-signature-ok'), ok: true, warn: false }
        : { text: t('admin-backup-signature-bad'), ok: false, warn: false }
})

const rows = computed(() => [
    { label: t('admin-backup-created'), value: new Date(meta.value.created_at).toLocaleString() },
    { label: t('admin-backup-master-version'), value: meta.value.master_version },
    { label: t('admin-backup-schema'), value: schema.value, bad: !props.verdict.schema_ok },
    { label: t('admin-backup-size'), value: total.value },
    { label: t('admin-backup-files'), value: String(meta.value.data_files) },
    {
        label: t('admin-backup-signature'),
        value: signature.value.text,
        bad: !signature.value.ok && !signature.value.warn,
        warn: signature.value.warn,
    },
])
</script>

<template>
    <div class="grid gap-4">
        <!-- Плитками, а не строками «слева подпись — справа значение»: карточка
             восстановления идёт во всю ширину, и такие строки растянуло бы к
             противоположным краям. -->
        <dl class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            <div
                v-for="row in rows"
                :key="row.label"
                class="rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] px-4 py-3"
            >
                <dt
                    class="text-[10px] font-black uppercase tracking-wider text-[var(--noro-muted)]"
                >
                    {{ row.label }}
                </dt>
                <dd
                    class="mt-1 truncate text-sm font-bold"
                    :class="
                        row.bad
                            ? 'text-[var(--noro-danger)]'
                            : row.warn
                              ? 'text-[var(--noro-amber)]'
                              : 'text-[var(--noro-text)]'
                    "
                >
                    {{ row.value }}
                </dd>
            </div>
        </dl>

        <NoroNote v-if="!verdict.parts_ok" icon="i-lucide-triangle-alert">
            <span v-for="p in verdict.problems" :key="p" class="block">{{ p }}</span>
        </NoroNote>

        <NoroNote v-if="refusal" icon="i-lucide-shield-x">{{ refusal }}</NoroNote>
    </div>
</template>
