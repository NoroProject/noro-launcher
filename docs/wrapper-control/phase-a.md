# Фаза A — каталог модов и единая установка

Статус: **сделано**. Дата: 2026-08-09.

Контекст для того, кто продолжит работу с нуля. Общий план всех фаз — в
[README.md](./README.md).

## Что было до

- Установка модов жила в `builds.rs` тремя почти одинаковыми функциями
  (`add_modrinth`, `add_curseforge`, `add_url`), каждая сама качала файл и сама
  писала `upsert_build_file`. Цель всегда одна — клиентская сборка.
- UI поиска: модалка `ModDatabaseModal.vue` поверх `ModrinthSearchTab.vue`. Одно
  поле, 30 хитов, без фасетов, сортировки, пагинации и карточки проекта.
  CurseForge — два поля для ручного ввода числовых id.
- `builds.rs` был 1067 строк при лимите 150.

## Что стало

### Мастер

Новый крейтовый модуль `crates/master/src/catalog/`:

| Файл | Роль |
|---|---|
| `mod.rs` | `fetch_json` с кешем, диспетчер по провайдеру, режим `all` (чередование выдач) |
| `cache.rs` | TTL-кеш ответов, 5 минут, потолок 512 записей |
| `types.rs` | Нормализованные `ModHit` / `ModProject` / `ModVersion` / `ModSource` / `ResolvedMod` |
| `query.rs` | `SearchQuery` (фасеты приходят строкой через запятую) и `SearchPage` |
| `json.rs` | Читалки полусвободного JSON (`text`, `opt_text`, `num`, `str_list`, `items`) |
| `modrinth.rs` / `modrinth_map.rs` | Запросы и разбор Modrinth |
| `curseforge.rs` / `curseforge_map.rs` | То же для CurseForge |
| `resolve.rs` | `ModSource` → файл в сторе + метаданные (`ResolvedMod`) |
| `tests.rs` | 10 тестов на фасеты и разбор чужого JSON |

Кеш висит в `AppState.catalog` (`HttpCache`), создаётся в `lib.rs`.

Новые ручки (`api/admin/catalog.rs`, маршруты в `admin/router.rs::catalog_router`):

```
GET  /api/admin/catalog/search?q&provider&project_type&mc&loader&categories&side&sort&offset&limit
GET  /api/admin/catalog/categories?provider&project_type
GET  /api/admin/catalog/providers
GET  /api/admin/catalog/{provider}/project/{id}
GET  /api/admin/catalog/{provider}/project/{id}/versions?mc&loader
POST /api/admin/mods/install
```

Все под `PERM_ADMIN_BUILDS`.

`POST /api/admin/mods/install` (`api/admin/mod_install.rs`) — единая установка:

```json
{ "source": {"kind":"modrinth","version_id":"QV48eyCs"},
  "targets": [ {"kind":"build","build_id":"…","optional":{…}},
               {"kind":"game_server","id":"…"},
               {"kind":"all_servers","server_id":"…"} ] }
```

Отвечает массивом `TargetResult` — по строке на цель. Источник резолвится один
раз, дальше веер. `all_servers` разворачивается в конкретные `game_server` до
установки (прокси отсеиваются), чтобы в отчёте была видна каждая машина.

Удалено из `builds.rs`: `search_mods`, `modrinth_project_versions`,
`add_modrinth`, `add_curseforge`, `add_url` и их маршруты (−300 строк, файл стал
766). Помощники `broadcast_builds_changed`, `build_server_id`,
`optional_from_draft`, `append_optional_mod` стали `pub(super)`;
`OptionalModDraft` переехал в секцию опциональных модов.

Заглушка `crates/master/src/wrapper/mod.rs::install_mod` возвращает «враппер не
подключён» — её замещает фаза D.

### Веб

| Файл | Роль |
|---|---|
| `types/catalog.ts` | Типы каталога и установки |
| `utils/catalog.ts` | `compactNumber`, `compactBytes`, `relativeDate`, `channelColor` |
| `composables/useModCatalog.ts` | Состояние поиска, фасеты, пагинация, дебаунс 400 мс |
| `composables/useModProject.ts` | Карточка проекта, версии, `fetchLatest` для установки в один клик |
| `composables/useInstallTargets.ts` | Сборки и игровые сервера пака, выбор целей |
| `composables/useModInstall.ts` | POST установки и разбор построчного отчёта |
| `components/mods/Browser.vue` | Сборка страницы: фасеты \| выдача \| панель проекта |
| `components/mods/Facets.vue` | Провайдер, тип, версия MC, лоадер, сторона, категории |
| `components/mods/SearchBar.vue` | Поиск (хоткей `/`) и сортировка |
| `components/mods/HitCard.vue` | Карточка выдачи |
| `components/mods/ProjectPanel.vue` | Версии / описание / галерея |
| `components/mods/VersionRow.vue` | Строка версии |
| `components/mods/Body.vue` | Описание через `marked` + `DOMPurify` |
| `components/mods/TargetPicker.vue` | Модалка выбора целей и карточки опционального мода |
| `components/mods/OptionalForm.vue` | Поля опционального мода |
| `components/build/ModCatalogPanel.vue` | Вход в каталог со страницы сборки |
| `pages/admin/mods.vue` | Глобальный браузер |
| `pages/admin/servers/[id]/build/[bid]/mods.vue` | Тот же браузер в контексте сборки |

Удалено: `ModDatabaseModal.vue`, `ModrinthSearchTab.vue`, `build/ModsPanel.vue`,
`composables/useBuildModActions.ts`.

`pages/admin/servers/[id]/build/[bid].vue` переехал в `[bid]/index.vue` — иначе
вложенный маршрут `[bid]/mods` превратил бы страницу сборки в layout с
`<NuxtPage/>`.

Стили `.noro-markdown` вынесены из `admin/MarkdownEditor.vue` в
`assets/css/markdown.css`: они нужны и там, где редактора на странице нет.

В сайдбар добавлен пункт **Mods** → `/admin/mods`.

## Решения, которые стоит знать

**Нормализация на мастере, а не в вебе.** Modrinth и CurseForge отдают одно и то
же разной формой. Если приводить их в компонентах, развилка «какой провайдер»
расползётся по всей админке.

**Режим `all` чередует выдачи, а не сортирует.** Релевантность Modrinth и
популярность CurseForge — разные шкалы, общий порядок из них не собрать.
Чередование даёт обоим равные шансы попасть в первый экран и объяснимо.

**Категории AND, а не OR.** Выбрав «magic» и «technology», админ ищет мод,
который и то и другое. Каждая категория уходит в свою группу фасетов.

**Категории у провайдеров несовместимы**: у Modrinth slug, у CurseForge числовой
id. Список грузится под выбранного провайдера; в режиме `all` берётся Modrinth, а
CurseForge молча игнорирует нечисловые значения.

**Автора берём из карточки поиска, не из карточки проекта.** Проверено вживую:
`GET /v2/project/{id}` у Modrinth **не отдаёт** поле `author`, а
`/members` не помечает владельца без авторизации (`is_owner: null`, роли —
свободный текст). Поэтому `mod_install` ставит значения из админки первыми, а
метаданные установки — вторыми, и `resolve.rs` превращает пустые строки в `None`.
Обратный порядок (как было в старом коде) затирал бы автора пустотой. Заодно в
старом коде в поле `author` писался `project["title"]` — это исправлено.

**Описание проекта санитайзится.** CurseForge отдаёт готовый HTML, написанный
посторонними людьми. Без `DOMPurify` это XSS в сессии администратора.

**CurseForge без ключа выключен целиком.** `/providers` говорит вебу, что
доступно; чипы гасятся, режим `all` откатывается на Modrinth.

## Проверено

- `cargo check --workspace` — чисто.
- `cargo test -p master catalog` — 10/10.
- `cd web && bun run build` — успешно.
- `cd web && bun run typecheck` — чисто (заодно починены три ошибки, которые
  висели до этого: `pendingAction` вместо `busy` на кнопке Apply Versions,
  `color="blue"` у `UProgress`, нетипизированный `auth.request` в
  `FileEditorModal`).
- Контракты Modrinth сверены живыми запросами: `search`, `project`, `version`,
  `tag/category` — все поля, которые читает маппер, существуют.

**Не проверено вживую:** установка через `POST /api/admin/mods/install` и
CurseForge-ветка — локально не поднят Postgres (докер не запущен) и нет
`CURSEFORGE_API_KEY` в окружении сессии.

## Что дальше

Фаза B — канал управления враппером. Цели `game_server` в пикере уже
отрисованы и отправляются на мастер; там их принимает заглушка
`wrapper::install_mod`. Фаза D делает её настоящей — переписывать UI не нужно.
