/** Находка сверки игрового каталога с манифестом. */
export interface IntegrityFlag {
  id: number
  at: string
  user_id: string
  server_id: string | null
  build_id: string | null
  build_version: string
  launcher_version: string
  kind: 'extra_file' | 'modified_file' | 'missing_file' | 'forbidden_optional_mod'
  /** Путь внутри инстанса либо имя мода. */
  subject: string
  detail: string | null
  /** Лаунчер убрал находку сам. */
  repaired: boolean
  reviewed_at: string | null
  reviewed_by: string | null
}

export const INTEGRITY_LABELS: Record<IntegrityFlag['kind'], string> = {
  extra_file: 'Extra file',
  modified_file: 'Modified file',
  missing_file: 'Missing file',
  forbidden_optional_mod: 'Optional mod without permission',
}
