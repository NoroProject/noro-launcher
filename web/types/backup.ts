/** Паспорт архива — то, что мастер кладёт в `meta.json` и подписывает. */
export interface BackupMeta {
    master_version: string
    schema_version: number
    created_at: string
    dump_bytes: number
    data_bytes: number
    data_files: number
    /** Без `NORO_SIGNING_KEY` мастеру нечем подписывать, и он это признаёт. */
    signed: boolean
}

/** Итог проверки архива. Причину отказа считает мастер, не админка. */
export interface Verdict {
    meta: BackupMeta
    signature_ok: boolean
    parts_ok: boolean
    problems: string[]
    known_schema_version: number
    schema_ok: boolean
}

export interface InspectRes {
    upload: string
    verdict: Verdict
    refusal: string | null
}

export interface RestoreRes {
    ok: boolean
    /** Куда уехали прежние файлы — относительно тома данных. */
    previous_data: string
    restarting: boolean
}
