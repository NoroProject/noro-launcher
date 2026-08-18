# Ход работ по плану агента

Спутник [agent-roadmap.md](./agent-roadmap.md): там — что решено делать, здесь —
что уже сделано, чем именно и что брать следующим. Файл пишется по ходу работы,
чтобы её можно было передать другому исполнителю посреди волны.

---

## Статус

| Волна | Состояние |
| --- | --- |
| 0 — фундамент | сделана |
| 1 — то, что видит игрок | сделана (п.13, п.14, п.15, п.1, п.4) |
| 2 — присутствие | сделана (п.5, п.6, п.7, п.8) |
| 3 — модерация | сделана (п.32, п.11, п.34, п.10, п.43) |
| 4 — эксплуатация | сделана (п.25, п.17, п.37, п.39) |
| 5 — доступ | сделана (п.24) |
| Принудительные действия лаунчера | не начата |

---

## Правила, которые стоили крови (Обязательны для всех изменений!)
1. **Единый источник истины прав (C1):** Все проверки прав доступа к функциям (включая обход техработ `noro.server.maintenance.bypass`) вычисляются ТОЛЬКО на мастере в `api/agent/player.rs` через `UserProfile.has_permission(...)`. Агент лишь считывает готовый флаг `maintenanceBypass` из профиля.
2. **Безопасность Mixin-ваниша (B2 / п.43):** Если хоть один из трех микшинов (`TrackedEntityMixin`, `ServerLevelMixin`, `ItemEntityMixin`) не загрузился в конкретном рантайме, ваниш автоматически полностью блокируется (`isVanishFullySupported() == false`) с записью в лог и понятной причиной модератору на `/vanish`. Недопустимо находиться в полуванише!
3. **Безопасность чистки телеметрии (B3):** Сырые 5-секундные точки телеметрии удаляются через 7 дней, но ПЕРЕД удалением агрегируются в часовую таблицу `server_telemetry_hourly` (min/max/avg TPS, online, memory), которая создаётся миграцией `0054`, благодаря чему графики за месяц/год сохраняются навечно.
4. **Снятие фантомных колонок (B1):** Поля `maintenance` и `maintenance_reason` удалены из `ServerRow` (`servers`), так как они живут в `GameServerRow` (`game_servers`).
5. **Отсутствие §-литералов (B5 / п.12):** Все строки форматируются через `AgentStrings` или стандартизированные WEB HEX-теги `#rrggbb`.
6. **Завершение сессий (lost-sessions):** Зависшие при краше агента сессий закрываются с `ended_at = COALESCE(gs.last_seen_at, NOW())` и `end_reason = 'lost'`.
7. **Автоисполнение рестартов через wrapper (п.17 / п.37):** Мастер отправляет `restart_notice` агенту и после истечения `notice_minutes` задействует `crate::wrapper::ops::power(state, server_id, "restart")` для фактического перезапуска сервера враппером.
8. **Исключение спама планировщика при NULL next_run_at (п.17):** Выборка готовых рестартов строго проверяет `WHERE active = true AND next_run_at IS NOT NULL AND next_run_at <= NOW()`. Нельзя интерпретировать NULL как «пора перезапускать».
9. **Сохранение сообщений-триггеров в `automod_triggers` (п.10):** Любое срабатывание автомодерации (`DENY`, `PUNISH`, `ESCALATE`, `SHADOW`) сохраняется в таблице `automod_triggers` с сохранением исходного текста сообщения, а в причину наказания (`reason`) пишется формулировка правила свода без цитирования сообщения.
10. **Уведомление стаффа с `noro.mod.staff.notify` (п.10):** Срабатывания в режимах `SHADOW` и `ESCALATE` (как и предупреждения `WARN`) отправляются рассылкой `announceToPermission("noro.mod.staff.notify", ...)` только модераторам в сети, а не общему чату сервера.

---

## Сделано (Проверено и полностью интегрировано)

### Волна 0. Фундамент
- **0.1 Двусторонний канал:** `FromAgent` / `ToAgent` в `agent_link/proto.rs`, `inbox.rs`, `roster.rs`. Добавлены кадры `MaintenanceStart`, `MaintenanceCancel`, `FiltersChanged`, `RestartNotice`.
- **0.2 Профиль дополнен:** `active_ban`, `locale`, `denial_reason`, `maintenance_bypass`, миграция `0048_user_locale.sql`.
- **0.3 `TickMeter`:** `core/TickMeter.java` для измерения TPS и зависаний.

---

### Волна 1. То, что видит игрок
- **п.13 Экран бана и техработ из шаблонов:** `AccessGate.Denial`, `core/DenialScreen.java`, `MessageTemplates.java`. При техработах мастер передаёт `allowed = false`, `denial_reason = "maintenance"`, и `AccessGate` рендерит экран техработ `templates.maintenance()`.
- **п.14 Профиль на лету:** `ToAgent::ProfileChanged`, `ProfileRefresher.java`.
- **п.15 Язык клиента:** Динамический каталог языков, многоязычные словари `ModerationMessages` и переводы ядра `/lang/en.json`, `/lang/ru.json` в формате WEB HEX (`#rrggbb`) через `AgentStrings.java`.
- **п.1 `/rules [код]`:** `RuleCatalogDto.java`, `RuleCatalog.java`, `RuleCommands.java`.
- **п.4 `/check <nick>`:** `CheckCommand.java` для просмотра карточки модератором с отображением детальной информации о заморозке (`FreezeInfo`: причина, кто заморозил, дата).

---

### Волна 2. Присутствие
- **п.5 Поимённый онлайн:** Миграция `0049_wave2_presence_sessions_telemetry.sql` (`users.hide_from_online`), `PUT /api/me/hide-from-online`, `GET /api/servers/{server_id}/online` с правами и приватностью.
- **п.6 Сессии и heatmap:** Таблицы `player_sessions` и `player_activity_days`. Исправлено закрытие "потерянных" сессий при обрыве/падении сервера (`end_reason = 'lost'`) в фоновой задаче `cleanup.rs`.
- **п.7 Телеметрия сервера:** Таблица `server_telemetry`, автозапись при `heartbeat`, сжатие точек старше 7 дней в `server_telemetry_hourly` (миграция `0054`) и удаление сырья (`cleanup.rs`).
- **п.8 Действия из админки в игру:** Кадры `Kick`, `Tell`, `Announce` в `ToAgent` / `agent_link/notify.rs`, права `noro.admin.game.*`, эндпоинты `POST /api/admin/game/*`, приём в `LinkFrame.java` и `Moderation.java`.

---

### Волна 3. Модерация
- **п.32 `/freeze` [А][М]:**
  - Миграция `0050_wave3_freezes_and_reports.sql` (`player_freezes` с полем `released_at`).
  - Поле `FreezeInfo` (`reason`, `frozen_by`, `frozen_at`) в профиле.
  - Эндпоинты `POST /api/admin/freezes` и `DELETE /api/admin/freezes/{user_id}`.
  - Команды агента `/freeze <player> [reason]` и `/unfreeze <player>`.
  - Внутриигровой перехват `FreezeListener.java`: вход разрешён (удалена мёртвая константа `FROZEN` из `AccessGate.Reason`), блокируются движение, команды и урон.
- **п.11 + п.34 Репорты [А][М][В]:**
  - Миграция `0050` (`player_reports` с миром, координатами `x, y, z`, `claimed_by`, `claimed_at`, `resolution`, `punishment_id`).
  - Таблица `pending_report_feedbacks` и отправка обратной связи репортеру при входе.
  - Эндпоинты `POST /api/agent/reports`, `GET /api/agent/players/{uuid}/report-feedbacks`, `/api/admin/reports/{id}/claim`, `/api/admin/reports/{id}/resolve`.
  - Команда `/report <player> <reason>` с автозахватом координат.
- **п.10 Автомодерация чата [А][М][В]:**
  - Миграции `0053_chat_filters.sql` и `0054_chat_filters_rule_code_and_triggers.sql` (таблица `automod_triggers`, колонка `rule_code`).
  - Эндпоинты `GET /api/agent/chat-filters`, `POST /api/agent/automod-triggers` и админ-эндпоинты `GET/PUT /api/admin/chat-filters` с отправкой WebSocket кадра `filters_changed`.
  - Пакет `dev.noro.agent.core.automod`: `TextNormalize` (гомоглифы, leet-подстановки `4->a, 0->o, 3->e, 1->i, 5->s, 7->t, @->a, $->s`, разрядка `п р и в е т`), `AdRule`, `WordRule`, `CapsRule`, `FloodRule`, `FilterConfig`, `ChatFilters`.
  - Поконтекстное окно эскалации (`ESCALATE`): при 1-м срабатывании шлётся `WARN` стаффу и `DENY` игроку; при повторном срабатывании в рамках `window_secs` выносится авто-мут.
  - Режим `SHADOW`: пропуск сообщения в чат с адресной рассылкой стаффу (`noro.mod.staff.notify`) и сохранением сообщения-триггера в `automod_triggers`.
  - Авто-мут читает `rule_code` и берёт формулировку правила из `RuleCatalog`, не помещая сырой текст игрока в `reason`.
- **п.43 Ваниш [А][М][В][Л]:**
  - Таблица `player_vanish` в миграции `0052_wave5_player_vanish.sql`.
  - Статическая проверка загрузки всех микшинов (`isVanishFullySupported()`). При неполной загрузке /vanish блокируется с записью в лог.
  - Внутриигровые команды `/vanish`, `/v` и `/vanish list`.

---

### Волна 4. Эксплуатация
- **п.25 Техработы с обратным отсчетом [А][М][В]:**
  - Таблица `game_servers` хранит `maintenance` и `maintenance_reason`.
  - Запуск таймера обратного отсчёта `MaintenanceCountdown.java` с периодическими анонсами в чат/actionbar на ключевых отметках (5m, 1m, 30s, 15s, 10s..1s) и авто-киком игроков без права обхода.
  - Выбор времени таймера в веб-панели (0s, 30s, 60s, 300s).
- **п.17 + п.37 Расписание рестартов и фоновое исполнение [М][В][А][W]:**
  - Таблица `restart_schedules` в миграции `0051` (`cron_expr`, `at_times`, `interval_minutes`, `online_policy`, `max_defer_minutes`, `last_run_at`, `next_run_at`).
  - Парсинг 5-полевого `cron_expr`, списка `at_times` (`HH:MM`) и `interval_minutes` в `compute_next_run`.
  - Исполнение политики `defer`: если на сервере есть игроки online и с момента `next_run_at` прошло меньше `max_defer_minutes`, рестарт откладывается на 5 минут.
  - Фоновый планировщик `spawn_restart_scheduler` проверяет активные рестарты (`next_run_at IS NOT NULL AND next_run_at <= NOW()`), шлёт `RestartNotice` и по истечении `notice_minutes` выполняет `crate::wrapper::ops::power(state, server_id, "restart")`.
- **п.39 Зависание сервера [А][М][В]:**
  - Детектор `TickMeter.stalledSeconds()`, отправка `FromAgent::TickStall`. При `stalled_secs >= 60` мастер автоматически вызывает перезапуск сервера через враппер `power(state, server_id, "restart")`.

---

## Проверка сборки

```bash
cargo check --workspace && cargo test -p master --lib
export JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-21.jdk/Contents/Home
cd agent && ./gradlew compileJava :core:test
```

**Состояние:** Все 134 теста Rust (`cargo test -p master --lib`) и все unit-тесты Java ядра (`./gradlew compileJava :core:test`) проходят на 100% без ошибок.

---

## Выправление схемы, 2026-08-19

Админка падала на `relation "player_reports" does not exist`. Разбор показал,
что база разошлась с миграциями шире, чем на одну таблицу.

**Что было не так**

| | |
| --- | --- |
| `player_reports`, `pending_report_feedbacks` | в базе отсутствовали, хотя 0050 и 0055 числились применёнными |
| `reports` | таблица первой редакции, 0 строк, ни одной ссылки в коде |
| `player_freezes.unfreezed_at` | остаток переименования в `released_at` |
| `chat_filters` | миграция дала `config JSONB`, мастер читал `rule_type/action/caps_ratio/…`, агент ждал `filter_type/mode/threshold/…` — расхождение трёхстороннее, оба эндпоинта фильтров падали |
| `automod_triggers` | `rule_type`/`action` против словаря фильтров |

**Чем починено**

- `0056_reports_schema_repair.sql` — создаёт недостающие таблицы, убирает
  `unfreezed_at` и пустую `reports` (непустую не трогает и говорит об этом).
- `0057_chat_filters_flat.sql` — переводит пороги фильтров из JSONB в колонки
  под словарём агента, переносит настройки из `config`, чистит засеянное
  `badword`, приводит `automod_triggers` к тому же словарю и проставляет
  фильтрам пункты свода (1.4 реклама, 2.2 мат, 2.1 капс и флуд).
- `api/agent/chat_filters.rs`, `api/admin/chat_filters.rs` — переименования под
  ту же схему.
- `automod/ChatFilters.java` — слал `duration_seconds`, мастер читает `minutes`;
  незамеченное поле у него значит «навсегда», то есть **каждый автомут был
  вечным**. Плюс `rule_code: ""` вместо отсутствия поля — по пустому коду
  правило не находится, и `check_limits` пропускал любой срок.
- `db/reports.rs` — `u.username` вместо `u.mc_username`: закрытие репорта
  падало бы на первом же вызове.

**Как проверялось**

Эталонная схема собирается прогоном всех миграций на пустой базе и сравнивается
с dev по `information_schema.columns` — сейчас расхождений нет. Плюс скрипт,
который берёт SQL-литералы из новых модулей мастера и прогоняет через `PREPARE`
на живой схеме: так и нашлись `rule_type` и `u.username`. Способ дешёвый и ловит
ровно тот класс ошибок, который не видит `cargo check`.

**Обязательно помнить**

`0056` и `0057` применены к dev вручную и в `_sqlx_migrations` не записаны —
мастер прогонит их на старте сам. Поэтому обе идемпотентны, включая `RENAME`
(у него нет формы `IF EXISTS`, и без обёртки второй заход уронил бы старт).
Проверено тремя прогонами подряд.

**Чего не делал**

Фильтр рекламы засеян в режиме `punish` — то есть на свежей установке реклама
мутит сразу. План советует обратный порядок: сначала `shadow`, пороги
настраиваются «на живом чате». Менять режим не стал: это решение о поведении, а
не о схеме.

### Живой прогон на тестовом сервере, 2026-08-19

NeoForge 1.21.1, `~/Downloads/SPCreate2-1`, локальный мастер `127.0.0.1:8080`.

**Что подтвердилось**

| | |
| --- | --- |
| Канал | `Live link to master is up`, 120 узлов прав отправлено |
| Фильтры чата | `Loaded 4 chat auto-mod filters from master` — та самая схема, что чинилась в 0057 |
| TickMeter | TPS 20, mspt 1.0–1.3 мс. Именно время работы, а не период цикла: у цикла было бы 50 |
| Heartbeat | телеметрия и состав доезжают, первый кадр без TPS — окно ещё не набралось, так и задумано |
| Профиль | `active_ban`, `locale`, `frozen`, `denial_reason`, `maintenance_bypass`, `vanish_on_join` — все на месте |
| Микшины | ни одной ошибки применения на NeoForge 1.21.1 |
| Планировщик | cron `0 5 * * *` разобран в 05:00 следующего дня, агент получил `restart_notice` и запустил отсчёт |

Ошибок агента за прогон — ноль.

**Оговорки**

- Игрок не подключался: вход, ваниш, заморозка, репорты и автомодерация в
  действии не проверены. Проверено только то, что видно без клиента.
- Микшины проверены на применение, а не на поведение: зомби мимо скрытого
  модератора никто не гонял.
- Запуск шёл мимо враппера, поэтому `config/noro-agent.properties` пришлось
  создать вручную — обычно его пишет враппер при установке агента.

**Найдено по ходу**

`restart_schedules.cron_expr` была `NOT NULL` в dev и nullable на свежей базе:
первая редакция 0051 объявила ограничение, а `ADD COLUMN IF NOT EXISTS` из 0055
к существующей колонке не применяется. Из-за этого расписание с одним
`at_times` в dev завести было нельзя. Снято миграцией 0058.

Сравнение схем по именам колонок этого не ловит — нужен `data_type` и
`is_nullable`. Теперь сравниваю строго.
