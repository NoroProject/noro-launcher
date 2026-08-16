/** Состояние первичной настройки. */
export interface SetupStatus {
  setup_completed: boolean
  /** Какие секреты видны мастеру в окружении. Значения не отдаются никогда. */
  secrets: Record<string, boolean>
  /**
   * Можно ли на этом адресе привязать passkey. WebAuthn не работает по http://
   * на не-localhost адресе, а инстанс на http://<ip>:8080 — типовой первый
   * запуск.
   */
  secure_context: boolean
}

/** Ключ подписи манифестов. Приватная часть показывается один раз. */
export interface SigningKey {
  private_hex: string
  public_hex: string
  env_line: string
}
