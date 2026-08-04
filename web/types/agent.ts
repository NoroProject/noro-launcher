/** Файл из `{NORO_DATA_DIR}/agents`: агент под платформу либо сам ServerWrapper. */
export interface AgentFile {
  file: string
  /** `paper`, `fabric`, `neoforge`, `forge` — либо `wrapper`. */
  platform: string
  /** `null` у враппера: он один на все версии. */
  mc_version: string | null
  size: number
  sha1: string
  url: string
}
