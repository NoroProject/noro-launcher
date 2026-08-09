# Фаза E — бэкапы серверной директории

Статус: **сделано**. Дата: 2026-08-09.

Общий план — в [README.md](./README.md), моды на сервере — [phase-d.md](./phase-d.md).

## Что появилось

### Враппер (Java)
- `ServerBackups`: создание, перечисление, восстановление и удаление zip-архивов в подкаталоге `noro-backups`.
- `BackupPacker`: фильтрация логов, временных кешей (`logs`, `crash-reports`, `cache`, `.git`, `noro-backups`) и самого конфига враппера с ключом доступа.
- `ServerBackupsTest`: юнит-тесты создания и восстановления архива.

### Мастер (Rust)
- Операции протокола `backup_create`, `backup_list`, `backup_restore`, `backup_delete` в `Op`.
- Хендлеры `wrapper_backups` и эндпоинты в `router.rs`:
  - `GET /api/admin/game-servers/{id}/backups`
  - `POST /api/admin/game-servers/{id}/backups`
  - `POST /api/admin/game-servers/{id}/backups/{name}/restore`
  - `DELETE /api/admin/game-servers/{id}/backups/{name}`

### Веб (Nuxt)
- Композибл `useServerBackups` для вызова операций снимков.
- Вкладка **Backups** в пульте управления игровым сервером (`GameserverBackups`).
