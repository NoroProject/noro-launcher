# Русский каталог. Плюрализация — родная для Fluent: селектор сам выбирает
# форму по правилам CLDR, для русского это one / few / many.

## Оконная рамка
lang-ru = RU
lang-en = ENG

## Навигация
nav-game = ИГРА
nav-mods = МОДЫ
nav-settings = НАСТРОЙКИ
nav-news = НОВОСТИ
nav-profile = ПРОФИЛЬ

## Боковая панель
sidebar-online = НА СВЯЗИ
sidebar-offline = НЕТ СВЯЗИ
sidebar-empty = Серверов пока нет. Добавьте сервер в админке.
sidebar-servers = { $count ->
        [one] { $count } сервер
        [few] { $count } сервера
       *[many] { $count } серверов
    }
sidebar-signed-out = Вход не выполнен
sidebar-no-discord = Discord не привязан

## Game
game-no-servers = Серверов нет
game-no-servers-hint = Добавьте сервер в админке, чтобы начать.

## Профиль
profile-title = ПРОФИЛЬ
profile-unavailable = Профиль недоступен.
profile-sign-out = Выйти
profile-cape = Плащ
profile-not-set = Не задан
profile-edit = Изменить
profile-upload-skin = Загрузить скин
profile-drag-to-rotate = Тяните, чтобы повернуть
profile-skin-loading = Загрузка...
profile-no-skin = Скин не установлен
profile-skin-untitled = Скин без названия
profile-skin-picker-failed = Не удалось открыть диалог выбора файла
profile-skin-unreadable = Не удалось прочитать файл
profile-skin-not-png = Это не PNG-изображение
profile-skin-too-large = Файл скина должен быть меньше 256 КБ
profile-presets-title = Пресеты скинов
profile-preset-new = Новый скин
profile-preset-upload-png = Загрузить .PNG
profile-preset-upload = Загрузить
profile-preset-current = Текущий
profile-preset-wear = Надеть
profile-cape-title = Выбор плаща
profile-cape-remove = Снять плащ
profile-cape-none = Без плаща
profile-cape-take-off = Снять

## Синхронизация и запуск
sync-checking = Проверка файлов...
sync-java = Загрузка Java...
sync-minecraft = Загрузка Minecraft...
sync-libraries = Загрузка библиотек...
sync-assets = Загрузка ресурсов...
sync-mods = Загрузка модов...
sync-forge = Применение патчей Forge...
sync-cleaning = Удаление лишних файлов...
sync-done = Готово
sync-files-left = { $count ->
        [one] остался { $count } файл
        [few] осталось { $count } файла
       *[other] осталось { $count } файлов
    }

## Ошибки и уведомления
error-game-exited = Игра завершилась с ошибкой
error-sign-in-cancelled = Вход отменён
error-background-failed = Не удалось загрузить фон сервера: { $reason }
retry = ПОВТОРИТЬ

## Уведомления от мастера
notif-server-error = Ошибка сервера: { $reason }
notif-no-server-access = Нет доступа к этому серверу
notif-no-published-build = У сервера нет опубликованной сборки
notif-build-files-restored = Восстановлены файлы сборки
notif-launch-blocked = Запуск заблокирован: в игровой папке найден запрещённый файл
notif-support-sent = Логи отправлены — спасибо
impersonate-title = ВХОД В АККАУНТ ИГРОКА
logreq-title = АДМИН ПРОСИТ ЛОГИ
notif-remote-action-done = Готово: { $detail }
remote-action-title = АДМИН ПРОСИТ ВЫПОЛНИТЬ ДЕЙСТВИЕ
remote-action-clear_asset_cache = Очистить кэш ассетов
remote-action-reinstall_build = Переустановить сборку
remote-action-restart_launcher = Перезапустить лаунчер
remote-action-verify_integrity = Проверить целостность
remote-action-accept = Разрешить
remote-action-decline = Отклонить
logreq-title-forced = ЛОГИ СОБРАНЫ ПО ЗАПРОСУ
logreq-not-collected = Миры, скриншоты и список серверов не собираются. Имя пользователя и токены вырезаются.
logreq-preview = Посмотреть, что отправится
logreq-send = Отправить
logreq-decline = Отклонить
logreq-close = Закрыть
impersonate-accept = Войти
impersonate-decline = Отклонить
impersonate-banner = Вы в аккаунте
impersonate-exit = Выйти
notif-impersonate-failed = Не удалось войти: { $reason }
notif-support-failed = Логи не отправились: { $reason }
notif-support-nothing-to-send = Логов пока нет: сначала запустите игру
notif-sign-in-first = Сначала войдите в аккаунт
notif-update-failed = Не удалось обновиться: { $reason }
notif-skin-upload-failed = Не удалось загрузить скин: { $reason }
notif-sign-in-to-upload = Для загрузки скина нужно войти
notif-sign-in-to-suggest = Для заявки на мод нужно войти
notif-already-running = Игра уже запущена

## Отказы авторизации
auth-banned = Аккаунт заблокирован
auth-session-expired = Сессия не найдена или истекла
auth-sign-in-first = Сначала войдите

## Вход
login-title = NORO LAUNCHER
login-subtitle = ВХОД ЧЕРЕЗ DISCORD
login-sign-in = ВОЙТИ ЧЕРЕЗ DISCORD
login-waiting = ОЖИДАНИЕ...
login-checking = ПРОВЕРКА СЕССИИ...
login-save-session = Запомнить сессию
login-auto-login = Входить автоматически
login-tagline = Лаунчер Minecraft серверов с автоматической установкой модов и скинов
login-sign-in-discord = Войти через Discord
login-sign-in-passkey = Войти через сайт (Passkey)

## Панель игры
game-build = СБОРКА
game-build-preview = ПРЕВЬЮ
game-start = ЗАПУСТИТЬ
game-install = УСТАНОВИТЬ
game-update = ОБНОВИТЬ
game-stop = ОСТАНОВИТЬ
game-preparing = ПОДГОТОВКА
game-locked = НЕТ ДОСТУПА
game-vip-only = ТОЛЬКО ДЛЯ ДРУЗЕЙ
toast-success = ГОТОВО
toast-warning = ВНИМАНИЕ
toast-error = ОШИБКА
toast-info = СООБЩЕНИЕ
game-node-offline = нет связи
game-online-unknown = онлайн неизвестен
sync-failed = ОШИБКА СИНХРОНИЗАЦИИ

## Новости
news-title = НОВОСТИ
news-empty = Новостей пока нет.
news-back = Назад
news-read = Читать

## Моды
mods-optional = ОПЦИОНАЛЬНЫЕ МОДЫ
mods-empty = Для этого сервера опциональных модов нет.

## Настройки
settings-title = НАСТРОЙКИ ЛАУНЧЕРА
settings-client-title = НАСТРОЙКИ КЛИЕНТА
settings-memory = ПАМЯТЬ JVM
settings-memory-default = ПАМЯТЬ JVM ПО УМОЛЧАНИЮ
settings-memory-hint = Сколько оперативной памяти выделяется Minecraft.
settings-jvm-flags = ФЛАГИ JVM
settings-jvm-hint = Дополнительные аргументы JVM при запуске.
settings-folder = ПАПКА
settings-folder-hint = Папка в которой находятся все файлы сборки.
settings-folder-open = Открыть
settings-console = КОНСОЛЬ ИГРЫ
settings-console-hint = Вывод логов игрового процесса.
settings-console-open = Открывать при запуске
settings-console-show = Показывать консоль при запуске игры
settings-fullscreen = ПОЛНОЭКРАННЫЙ РЕЖИМ
settings-fullscreen-hint = Запуск игры в полноэкранном режиме.
settings-fullscreen-show = Запускать в полноэкранном режиме
settings-reset = СБРОС
settings-update = ОБНОВЛЕНИЕ ЛАУНЧЕРА
settings-install-update = Установить обновление
settings-source-default = ПО УМОЛЧАНИЮ
settings-source-override = ЛОКАЛЬНАЯ НАСТРОЙКА
settings-source-recommended = РЕКОМЕНДАЦИЯ МАСТЕРА
settings-server = Сервер
console-title = КОНСОЛЬ ИГРЫ
settings-update-hint = Доступна новая версия лаунчера.
settings-crash-reports = ОТЧЁТЫ О ПАДЕНИЯХ
settings-crash-reports-hint = Отправлять анонимный отчёт, когда лаунчер падает. Ни аккаунт, ни имя компьютера в него не попадают. Применится после перезапуска.
settings-support-bundle = СООБЩИТЬ О ПРОБЛЕМЕ
settings-support-bundle-hint = Отправить логи последней игры администраторам. Миры, скриншоты и список серверов не собираются, а имя пользователя и токены вырезаются.
settings-support-send = Отправить логи

# ---------------------------------------------------------------------------
# Сайт. Префикс `web-` отделяет строки сайта от строк лаунчера: по нему
# фильтрует редактор в админке, а лаунчер эти ключи никогда не спрашивает.
# ---------------------------------------------------------------------------

## Навигация и подвал
web-nav-home = Главная
web-nav-servers = Серверы
web-nav-rules = Правила
web-nav-cabinet = Кабинет
web-nav-sign-in = Войти
web-nav-menu-open = Открыть меню
web-nav-menu-close = Закрыть меню
web-footer-tagline = Модовый проект Minecraft
web-footer-legal = © { $year } Noro. Не связан с Mojang и Microsoft.

## Главная
web-home-online-now = { $count ->
        [one] сейчас { $count } игрок в сети
        [few] сейчас { $count } игрока в сети
       *[many] сейчас { $count } игроков в сети
    }
web-home-lead = Модовый проект Minecraft с удобным лаунчером: авто-скачивание модов, вход через Discord, кастомные скины и плащи — всё в одном месте.
web-home-cta-cabinet = ОТКРЫТЬ КАБИНЕТ
web-home-cta-discord = НАЧАТЬ ЧЕРЕЗ DISCORD
web-home-stat-online = В сети
web-home-stat-servers = Серверов
web-home-worlds-eyebrow = Миры
web-home-worlds-title = Серверы проекта
web-home-worlds-all = Все серверы
web-home-feature-builds-title = Быстрая установка модов
web-home-feature-builds-text = Лаунчер сам загрузит и обновит все нужные моды и файлы — вам остаётся только нажать «Играть».
web-home-feature-identity-title = Единый профиль
web-home-feature-identity-text = Вход в один клик через Discord, скин, плащ и личный кабинет — всё привязано к вашему аккаунту.
web-home-feature-rules-title = Честная игра
web-home-feature-rules-text = Прозрачные правила и открытая модерация: правила всегда под рукой и понятны каждому.
web-home-flow-eyebrow = Путь до игры
web-home-flow-title = От Discord до Minecraft за четыре шага
web-home-step-signin-title = Авторизуйтесь
web-home-step-signin-text = Войдите через Discord на сайте — ваш аккаунт сразу готов к игре.
web-home-step-server-title = Выберите сервер
web-home-step-server-text = Выберите понравившуюся сборку и сервер в списке.
web-home-step-sync-title = Авто-скачивание
web-home-step-sync-text = Лаунчер сам загрузит и обновит все нужные файлы.
web-home-step-play-title = Запускайте и играйте
web-home-step-play-text = Нажмите кнопку запуска — Minecraft откроется с настроенными модами.
web-home-rules-title = Ознакомьтесь с правилами проекта
web-home-rules-text = Правила просты и понятны, а их соблюдение гарантирует комфортную и приятную игру для каждого.
web-home-rules-cta = Открыть правила

## Серверы
web-servers-meta-title = Серверы — Noro
web-servers-meta-description = Модовые серверы Minecraft проекта: версии, адреса и онлайн в реальном времени.
web-servers-title = Серверы
web-servers-lead = Все миры проекта с версией, адресом и текущим онлайном.
web-servers-online = Игроков в сети
web-servers-refresh = Обновить
web-servers-loading = Загружаем серверы…
web-servers-empty-title = Серверов пока нет
web-servers-empty-text = Проект ещё не открыл ни одного мира. Загляните позже.
web-servers-empty-short = Проект ещё не открыл ни одного мира.

## Карточка сервера
web-server-online = В сети
web-server-offline = Не в сети
web-server-players = Игроки
web-server-rules = Правила
web-server-address-copied = Адрес скопирован

## Скачивание лаунчера
web-download-title = СКАЧАТЬ ЛАУНЧЕР
web-download-lead = Войдите через Discord, выберите сервер — остальное лаунчер синхронизирует сам.
web-download-loading = Загружаем сборки…
web-download-none = Сборок лаунчера пока нет.
web-download-for = Скачать для { $platform }
web-download-other = Другие системы
web-download-signed = Каждая сборка подписана — лаунчер проверяет подпись перед запуском.

## Правила
web-rules-meta-title = Правила — Noro
web-rules-meta-description = Правила проекта: что можно на серверах, а что — нет, и что за это бывает.
web-rules-title = Правила
web-rules-lead = Каждое наказание ссылается на пункт правил. Ищите по коду, заголовку или тексту — те же коды используются в игре, в банах и в тикетах.
web-rules-total = Пунктов в своде
web-rules-search = Поиск по коду (1.1), заголовку или тексту
web-rules-search-aria = Поиск по правилам
web-rules-scope = Свод
web-rules-scope-general = Общий
web-rules-found = { $count ->
        [one] { $count } пункт по запросу «{ $query }»
        [few] { $count } пункта по запросу «{ $query }»
       *[many] { $count } пунктов по запросу «{ $query }»
    }
web-rules-loading = Загружаем правила…
web-rules-failed-title = Правила недоступны
web-rules-failed-text = Мастер-сервер не ответил. Попробуйте через минуту.
web-rules-nomatch-title = Ничего не нашлось
web-rules-nomatch-text = Ни один пункт не упоминает «{ $query }». Попробуйте код вида 1.1 или одно слово.
web-rules-empty-title = Правил пока нет
web-rules-empty-text = Команда ещё не опубликовала свод. Загляните позже.
web-rules-other = Прочие правила
web-rules-contents = Содержание
web-rules-count = { $count ->
        [one] { $count } пункт
        [few] { $count } пункта
       *[many] { $count } пунктов
    }
web-rules-copy-link = Скопировать ссылку на пункт
web-rules-link-copied = Ссылка скопирована
web-rules-link-copied-body = Правило { $code }

## Наказания, допустимые правилом
web-sanction-warn = Предупреждение
web-sanction-mute = Мут
web-sanction-ban = Бан
web-sanction-server-ban = Бан на сервере
web-sanction-possible = Возможное наказание
web-sanction-exact = { $kind } { $min }
web-sanction-range = { $kind } { $min }–{ $max }
web-sanction-from = { $kind } от { $min } до навсегда
web-sanction-upto = { $kind } до { $max }
web-sanction-any = { $kind } на любой срок или навсегда

## Кабинет: наказания
cabinet-punishments-title = Наказания и предупреждения
cabinet-punishments-lead = Полный список всех выдававшихся варнов, глобальных блокировок и ограничений по серверам
cabinet-punishments-refresh = Обновить
cabinet-punishments-none-title = Наказаний нет
cabinet-punishments-none-text = У вас нет активных или прошлых предупреждений и блокировок.

## Статусы и плашки наказаний
punishment-kind-ban = БАН
punishment-kind-warn = ВАРН
punishment-kind-mute = МУТ
punishment-kind-server-ban = СЕРВЕР БАН
punishment-status-active = ДЕЙСТВУЕТ
punishment-status-expired = ИСТЕКЛО
punishment-status-revoked = ОТМЕНЕНО
punishment-actor = Выдал: { $actor }
punishment-until = до { $date }
punishment-forever = навсегда

## Панель наказаний в админке
admin-punish-title = Наказания
admin-punish-empty = Записей не найдено.
admin-punish-revoked = отменено
admin-punish-expired = истекло
admin-punish-allowed-for-rule = Разрешено этим правилом
admin-punish-none-allowed = В этом правиле не задано допустимых наказаний.
admin-punish-kind = Вид
admin-punish-target-server = Целевой сервер
admin-punish-target-server-optional = (опционально)
admin-punish-target-all-servers = Все серверы
admin-punish-target-select-server = Выберите сервер…
admin-punish-reason = Причина
admin-punish-reason-placeholder = Пояснение причины выдачи наказания
admin-punish-bypass-hint = байпас: ограничения правила к вам не применяются
admin-punish-rule-label = Правило
admin-punish-rule-clear = Очистить правило
admin-punish-rule-search = Поиск по коду или тексту правила
admin-punish-rule-empty = Свод правил пуст.
admin-punish-rule-nomatch = Ничего не найдено.
admin-punish-duration-label = Срок
admin-punish-duration-hint-empty = пусто = навсегда
admin-punish-duration-hint-until = { $duration } — до { $until }

## Навигация и сайдбар
nav-admin-control = Управление проектом
nav-player-cabinet = Личный кабинет
nav-switch-to-cabinet = Личный кабинет
nav-switch-to-admin = Админ панель
nav-cabinet-home = Кабинет
nav-cabinet-skin = Скин
nav-cabinet-punishments = Наказания
nav-cabinet-rules = Правила
nav-cabinet-apps = Приложения
nav-cabinet-settings = Настройки
nav-group-management = Управление
nav-group-content = Контент
nav-group-system = Система

## Заметки к пользователю в админке
admin-notes-title = Заметки
admin-notes-subtitle = Только для администрации — игрок не видит эти записи.
admin-notes-placeholder = Что произошло…
admin-notes-add = Добавить
admin-notes-empty = Заметок пока нет.

## Главная страница кабинета
cabinet-title = Кабинет
cabinet-subtitle = Профиль и доступ
cabinet-player = Игрок
cabinet-mc-name = Игровой ник Minecraft
cabinet-mc-name-hint = Отображается в игре другим игрокам. До 16 символов.
cabinet-save = Сохранить
cabinet-profile-updated = Профиль обновлён
cabinet-passkeys-title = Ключи доступа Passkeys (WebAuthn)
cabinet-passkeys-lead = Беспарольный вход через Touch ID, Face ID или аппаратные ключи безопасности
cabinet-passkeys-add = Добавить Passkey
cabinet-passkeys-created = Создан { $date }
cabinet-passkeys-used = вход { $date }
cabinet-passkeys-unused = ни разу не использован
cabinet-passkeys-none-title = Нет привязанных ключей Passkey
cabinet-passkeys-none-text = Добавьте ключ Touch ID или Face ID для быстрой авторизации без Discord
cabinet-launcher-title = Лаунчер
cabinet-roles-title = Роли — { $count }
cabinet-perms-count = { $count } прав
cabinet-roles-none-title = Ролей пока нет
cabinet-roles-none-text = Доступ к серверам предоставляется через роли.
cabinet-direct-perms-title = Прямые права — { $count }
cabinet-direct-none-title = Напрямую ничего не выдано
cabinet-direct-none-text = Это нормально — права обычно приходят от ролей.

## Подключённые приложения
cabinet-apps-title = Подключённые приложения
cabinet-apps-subtitle = Управление приложениями с доступом к аккаунту
cabinet-apps-lead = Сторонние сервисы и лаунчеры, у которых есть доступ к вашему профилю
cabinet-apps-refresh = Обновить
cabinet-apps-default-desc = Доступ к вашему профилю Noro Network
cabinet-apps-revoke = Отозвать доступ
cabinet-apps-none-title = У вас нет подключённых сторонних приложений
cabinet-apps-none-text = Здесь будут отображаться приложения и лаунчеры, которым вы разрешили доступ

## OAuth2 Авторизация
oauth-loading-app = Загрузка информации о приложении...
oauth-official-app = Официальное приложение
oauth-app-access-request = Приложение запрашивает доступ к вашему аккаунту Noro Network.
oauth-discord-connected = Discord: Подключён
oauth-requested-permissions = Запрашиваемые разрешения
oauth-scope-profile = Профиль пользователя и игровой ник ({ $scope })
oauth-deny = Отклонить
oauth-allow = Разрешить доступ

## Менеджер скинов и плащей
skin-title = Скины и плащи
skin-subtitle = Менеджер скинов в стиле Modrinth & Pandora
skin-3d-character = 3D Персонаж
skin-custom-badge = Свой скин
skin-default-badge = По умолчанию
skin-reset-default = Сбросить на стандартный
skin-your-skins = Ваши скины и пресеты
skin-drop-hint = Нажмите на карточку «+» или перетащите файл для создания пресета
skin-new-skin = Новый скин
skin-upload-png = Загрузить файл .PNG
skin-equipped = Надет
skin-equip = Надеть
skin-official-skins = Официальные скины Minecraft
skin-mojang-desc = Стандартные персонажи Mojang
skin-available-capes = Доступные плащи
skin-pick-cape-desc = Выберите плащ для вашего персонажа
skin-capes-count = { $count } плащей
skin-no-cape = Без плаща
skin-no-capes-title = Нет доступных плащей
skin-no-capes-desc = У вас пока нет доступных плащей. Запросите доступ у администрации.

## Страница входа
login-secure-login = Безопасная авторизация
login-discord = ВОЙТИ ЧЕРЕЗ DISCORD
login-passkey = ВОЙТИ ЧЕРЕЗ PASSKEY
login-recovery-toggle = Использовать код восстановления
login-username-placeholder = Имя пользователя
login-submit = ВОЙТИ
login-recovery-hint = Каждый код работает один раз. После входа привяжите Passkey для быстрых входов в будущем.

## Админка: список блокировок
admin-blocklist-title = Список блокировок
admin-blocklist-subtitle = Файлы, запрещённые к нахождению в папке игры
admin-blocklist-note = Маска или хэш отсеивают простые моды. Список передаётся внутри подписанного манифеста, поэтому его нельзя подменить на клиенте.
admin-blocklist-mask = Маска имени
admin-blocklist-sha1 = Хэш SHA1
admin-blocklist-reason = Причина
admin-blocklist-action = Действие
admin-blocklist-act-delete = Удалять
admin-blocklist-act-flag = Только помечать
admin-blocklist-act-block = Блокировать запуск
admin-blocklist-empty-title = Ничего не заблокировано
admin-blocklist-empty-text = Добавьте маску или хэш — правила передаются внутри подписанного манифеста.

## Админка: модалки правил и категорий
admin-rule-edit = Редактирование правила { $code }
admin-rule-new = Новое правило
admin-rule-code = Код
admin-rule-section = Раздел
admin-rule-no-section = Без раздела
admin-rule-server-only = Только { $name }
admin-rule-wording = Формулировка
admin-rule-create = Создать правило
admin-cat-edit = Редактирование раздела { $name }
admin-cat-new = Новый раздел
admin-cat-number = Номер
admin-cat-parent = Родительский раздел
admin-cat-top = Верхний уровень
admin-cat-intro = Вводный текст (необязательно)
admin-cat-create = Создать раздел
admin-sanc-title = Допустимые наказания
admin-sanc-add-option = Вариант
admin-sanc-no-limits = Ограничения не заданы: наказать по этому правилу сможет только модератор с правом noro.mod.punish.bypass.
admin-sanc-kind = Вид
admin-sanc-from = От
admin-sanc-to = До
admin-sanc-note = Заметка (необязательно)

## Админка: версионирование лаунчера
admin-launchver-plat = Платформа
admin-launchver-kind = Тип
admin-launchver-curr = Действующая
admin-launchver-builds = { $count } сборок
admin-launchver-deploy-core = Выкатить core
admin-launchver-deploy-boot = Выкатить bootstrap
admin-launchver-is-curr = текущая
admin-launchver-stored = сохранена
admin-launchver-deploy = Выкатить

## Админка: переводы
admin-i18n-title = Переводы
admin-i18n-subtitle = Тексты лаунчера и интерфейса
admin-i18n-changed-count = { $count } из { $total } изменены
admin-i18n-hint = Оставьте поле пустым, чтобы использовать встроенный текст. На лаунчеры отправляются только ваши переопределения, поэтому тронутые ключи продолжат работать после обновлений.
admin-i18n-search-placeholder = Поиск ключа или текста
admin-i18n-only-changed = Только изменённые
admin-i18n-col-key = Ключ
admin-i18n-col-builtin = Встроенный
admin-i18n-col-override = Переопределение
admin-i18n-no-match = Ничего не найдено по фильтру.
admin-i18n-unsaved = Несохранённые изменения
admin-i18n-reset-all = Сбросить все переопределения

## Админка: настройки
admin-settings-title = Настройки
admin-settings-subtitle = Конфигурация инстанса
admin-settings-export-env = Экспорт .env
admin-settings-secrets-title = Секреты
admin-settings-secrets-lead = Только для чтения. Секреты задаются исключительно в переменных окружения.
admin-settings-secret-set = задан в окружении
admin-settings-secret-unset = не задан
admin-set-label-instance_name = Имя инстанса
admin-set-label-public_url = URL публичного API
admin-set-hint-public_url = Попадает во все манифесты, которые скачивает игрок.
admin-set-label-web_url = URL сайта
admin-set-hint-web_url = Passkey ключи бессрочно привязываются к этому домену.
admin-set-label-allowed_origins = Разрешённые CORS origins
admin-set-hint-allowed_origins = Через запятую. Пустое значение означает доступ для любого origin.
admin-set-label-discord_client_id = Discord Client ID
admin-set-label-files_cdn_url = CDN URL для файлов
admin-set-label-github_repo = Репозиторий GitHub
admin-set-label-github_ref = Ветка GitHub
admin-set-label-launcher_repo = Локальный исходник лаунчера
admin-set-from-env = из env
admin-set-from-env-title = Задано через { $env }. Переменные окружения приоритетнее БД.
admin-diag-panel-desc = Параметры, которые иначе видны только первой строчкой в логе старта.

## Админка: поддержка и логи
admin-support-title = Поддержка и логи
admin-support-subtitle = Полученные дампы и запросы сбора логов
admin-support-bundles-title = Полученные дампы ({ $count })
admin-support-archives-count = { $count } архивов
admin-support-user = Игрок { $id }
admin-support-voluntary = Добровольный
admin-support-forced = Принудительный
admin-support-date = Получен { $at } · Истёк { $expires }
admin-support-download-zip = Скачать ZIP
admin-support-nobundles-title = Дампов логов нет
admin-support-nobundles-text = Дампы поддержки от клиентов пока не поступали.
admin-support-requests-title = Запросы логов ({ $count })
admin-support-requests-count = { $count } запросов
admin-support-norequests-title = Запросов нет
admin-support-norequests-text = Запросы на сбор логов пока не отправлялись.

## Админка: роли и права
admin-roles-title = Роли
admin-roles-subtitle = Управление доступом через шаблоны прав
admin-roles-new-role = Новая роль
admin-roles-col-role = Роль
admin-roles-col-perms = Права
admin-roles-col-default = По умолчанию
admin-roles-yes = да
admin-roles-no = нет
admin-roles-modal-title = НОВАЯ РОЛЬ
admin-roles-modal-subtitle = Группа прав для пользователей лаунчера
admin-roles-name = Имя (ID)
admin-roles-display-name = Отображаемое имя
admin-roles-color = Цвет
admin-roles-order = Порядок сортировки
admin-roles-is-default = Роль по умолчанию
admin-role-back = Назад
admin-role-icon-hint = Один символ рядом с именем. Отображается в кабинете, лаунчере и игровом чате.
admin-role-inherits-label = Наследует от
admin-role-inherits-none = Ничего — только собственные права
admin-role-inherits-hint = Все права родительской роли автоматически действуют и здесь вверх по цепочке.
admin-role-lp-label = Группа LuckPerms
admin-role-lp-hint = Связывает роль с игровой группой. Оставьте пустым, если роль не предназначена для сервера.
admin-role-perms-title = Права роли
admin-role-perms-subtitle = Выдаются всем участникам этой роли.

## Админка: новости
admin-news-title = Новости
admin-news-subtitle = Новости и посты для лаунчера в Markdown
admin-news-new-post = Новый пост
admin-news-pinned = закреплена
admin-news-empty-title = Новостей пока нет
admin-news-empty-text = Создайте первый пост через кнопку вверху.
admin-news-modal-title = НОВЫЙ ПОСТ
admin-news-modal-subtitle = Публикация новости лаунчера в формате Markdown
admin-news-post-title = Заголовок
admin-news-post-body = Содержание
admin-news-post-pinned = Закреплена
admin-news-preview = Предпросмотр

## Админка: версионирование и сборка лаунчера
admin-launch-title = Лаунчер
admin-launch-subtitle = Версии, сборки тегов GitHub и деплой
admin-launch-build-tag = Собрать тег
admin-launch-build-log = Лог сборки
admin-launch-empty-title = Версий пока нет
admin-launch-empty-text = Соберите тег лаунчера с помощью кнопки вверху.
admin-launch-modal-title = СБОРКА GITHUB
admin-launch-modal-subtitle = Сборка релизного тега лаунчера
admin-launch-check-release = Проверить последний релиз

## Админка: токены API и CLI
admin-tokens-title = Токены API
admin-tokens-subtitle = Токены доступа для CLI и CI
admin-tokens-new-token = Новый токен
admin-tokens-secret-once = Секрет показывается только один раз
admin-tokens-last-used = Последнее использование
admin-tokens-empty-title = Токенов пока нет
admin-tokens-empty-text = Создайте токен CLI с помощью кнопки вверху.
admin-tokens-modal-title = НОВЫЙ ТОКЕН
admin-tokens-modal-subtitle = Секрет будет показан только один раз
admin-tokens-perms-label = Права (по одному на строку)

## Модерация пользователей
admin-users-version = Версия
admin-users-login-as = Войти от имени
admin-users-req-logs = Запросить логи
admin-users-end-sessions = Завершить все сессии
admin-users-impersonate-title = Войти от имени игрока
admin-users-impersonate-warn = Игрок не уведомляется. Все действия, совершённые вами в его профиле, фиксируются в журнале аудита под вашим именем.
admin-users-impersonate-code = Код восстановления
admin-users-impersonate-confirm = Подтвердить личность
admin-users-impersonate-request = Запросить доступ к { $name }
admin-users-reqlogs-title = Запрос логов
admin-users-reqlogs-target = Целевой сервер / сборка
admin-users-reqlogs-auto = Авто (Текущий / Корень лаунчера)
admin-users-reqlogs-why = Причина
admin-users-reqlogs-why-hint = Игрок увидит этот текст, и он останется в журнале аудита.
admin-users-reqlogs-force = Собирать без запроса.
admin-users-reqlogs-force-hint = Записывается отдельно в журнале аудита.
admin-users-reqlogs-btn-now = Собрать сейчас — { $name }
admin-users-reqlogs-btn-ask = Запросить — { $name }

## Общие веб-ключи и наказания
web-rules-cancel = Отмена
web-rules-save = Сохранить
web-rules-status-online = Онлайн
web-rules-status-offline = Офлайн
punish-warn = Предупреждение
punish-mute = Мут
punish-ban = Бан
punish-server-ban = Серверный бан

## Боковое меню админки
nav-admin-dashboard = Обзор
nav-admin-clients = Серверы
nav-admin-servers = Серверы
nav-admin-mods = Моды
nav-admin-users = Игроки
nav-admin-capes = Плащи
nav-admin-roles = Роли
nav-admin-integrity = Целостность
nav-admin-blocklist = Чёрный список
nav-admin-news = Новости
nav-admin-rules = Правила
nav-admin-translations = Переводы
nav-admin-wrapper = Обёртка
nav-admin-launcher = Лаунчер
nav-admin-tokens = Токены
nav-admin-audit = Журнал аудита
nav-admin-support = Логи поддержки
nav-admin-settings = Настройки

## Сборки и сервера в админке
admin-gs-title = Игровые сервера
admin-gs-subtitle = Инстансы этой сборки. Бэкенды передают онлайн и запускают агент; прокси — точка подключения.
admin-gs-add = Добавить сервер
admin-gs-host = Хост
admin-gs-port = Порт
admin-gs-backend = Бэкенд
admin-gs-proxy = Прокси
admin-gs-create = Создать
admin-gs-empty-title = Игровые сервера не зарегистрированы
admin-gs-empty-text = Добавьте сервер, чтобы получить секрет агента и видеть онлайн.
admin-gs-never-seen = ни разу не подключался
admin-gs-just-now = только что
admin-gs-ago = { $time } назад
admin-gs-no-address = без адреса
admin-gs-control-tooltip = Управление: консоль, файлы, моды
admin-gs-rotate-tooltip = Выдать новый секрет — старый перестанет работать
admin-set-profile-title = Профиль
admin-set-profile-text = Название сборки и место её отображения.
admin-set-client-title = Настройки клиента по умолчанию
admin-set-client-text = Версия и загрузчик модов для новых сборок.
admin-set-active = Активна
admin-set-active-hint = Показывать эту сборку в списке лаунчера.
admin-set-limited = Ограниченный доступ
admin-set-limited-hint = Требовать роль доступа для подключения игроков.
admin-set-client-defaults-hint = Сборки наследуют эти значения при создании. Адреса серверов указываются в игровых серверах ниже.
admin-builds-title = Сборки
admin-builds-subtitle = Управление версиями игры, файлами, модами и публикацией.
admin-builds-col-build = Сборка
admin-builds-col-status = Статус
admin-builds-published = опубликована
admin-builds-draft = черновик
admin-builds-empty-title = Сборок пока нет
admin-builds-empty-text = Создайте первую сборку на панели выше.

## Ресурсы сервера
admin-media-title = Ресурсы сервера
admin-media-subtitle = Иконка и баннер для карточки в лаунчере.
admin-media-banner-upload = Загрузить баннер
admin-media-banner-hint = Перетащите широкое изображение или нажмите для выбора.
admin-media-icon = Иконка
admin-media-icon-upload = Загрузить иконку

## Панели управления сборкой
admin-optmods-title = Опциональные моды
admin-optmods-configured = { $count } модов настроено
admin-optmods-allow-suggestions = Разрешить заявки на моды
admin-optmods-save = Сохранить опциональные моды
admin-optmods-empty-title = Опциональных модов нет
admin-optmods-empty-text = Игроки смогут включать дополнительные компоненты.
admin-optmods-untitled = Опциональный мод без названия
admin-optmods-display-name = Отображаемое имя
admin-optmods-category = Категория
admin-optmods-author = Автор
admin-optmods-icon-url = URL иконки
admin-optmods-files-csv = Файлы (через запятую)
admin-optmods-behavior = Поведение
admin-optmods-enabled-default = Включён по умолчанию
admin-optmods-visible = Видим в интерфейсе
admin-optmods-restricted = Ограниченный доступ
admin-paths-title = Правила путей
admin-paths-subtitle = Исключения синхронизации и переопределения
admin-paths-rules-count = { $count } правил
admin-paths-ignored = Игнорируемые
admin-paths-ignored-hint = Не затрагиваются синхронизацией
admin-paths-user-overrides = Пользовательские
admin-paths-user-overrides-hint = Загружаются один раз, правки сохраняются
admin-paths-edit-fm = Редактировать в файловом менеджере
admin-recom-title = Рекомендуемые настройки клиента
admin-recom-subtitle = Значения по умолчанию для лаунчера этой сборки
admin-recom-min-mem = Мин. память
admin-recom-max-mem = Макс. память
admin-recom-show-console = Показывать окно консоли при запуске
admin-recom-jvm-flags = Флаги JVM
admin-recom-save = Сохранить рекомендации

## Фильтры каталога и правила синхронизации
admin-facets-source = Источник
admin-facets-both = Оба
admin-facets-runs-on = Где работает
admin-facets-any-loader = Любой лоадер
admin-facets-any-version = Любая версия
admin-facets-any-side = любой
admin-facets-clear = Сбросить { $count } фильтров
admin-syncmodal-title = Редактирование правил синхронизации
admin-syncmodal-subtitle = Ручное редактирование путей для игнорирования или сохранения пользовательских изменений
admin-syncmodal-ignored-paths = Игнорируемые пути (Unmanaged)
admin-syncmodal-user-overrides = Пользовательские (User Managed)
admin-syncmodal-quick-add = Быстрое добавление:
admin-syncmodal-ignored-hint = Файлы и папки из этого списка никогда не будут загружаться или удаляться при синхронизации. Папки должны заканчиваться на /
admin-syncmodal-user-hint = Файлы из этого списка загружаются один раз при первой установке и никогда не перезаписываются.
admin-syncmodal-save = Сохранить правила

## Файлы и публикация сборки, установка обёртки
admin-filesummary-title = Файлы сборки
admin-filesummary-count = { $count } файлов
admin-filesummary-hidden = (основные ресурсы скрыты из превью)
admin-filesummary-open = Открыть файловый менеджер
admin-filesummary-empty = Файлов пока нет. Используйте панели справа для добавления.
admin-publish-title = Публикация сборки
admin-publish-subtitle = Загрузчик и подпись
admin-publish-draft = Режим черновика
admin-publish-live = Опубликовано
admin-publish-btn = Опубликовать сборку
admin-publish-rebuild = Собрать заново
admin-publish-scratch = Собрать с нуля
admin-publish-revert = Вернуть в черновики
admin-publish-delete = Удалить сборку
admin-wrapper-setup-title = Настройка игрового сервера
admin-wrapper-setup-step1 = 1. Скачайте wrapper.jar
admin-wrapper-setup-step1-hint = Положите файл рядом с jar-файлом вашего сервера.
admin-wrapper-setup-step2 = 2. Создайте noro-wrapper.properties
admin-wrapper-setup-step3 = 3. Запустите сервер через wrapper
admin-wrapper-setup-step4 = 4. Оставьте online-mode включённым

## Пользовательские настройки и поддержка
cabinet-settings-account = Аккаунт
cabinet-settings-session = Сессия
cabinet-settings-signout-hint = Выход из аккаунта затрагивает только этот браузер. В лаунчере сохранится своя сессия.
admin-support-panel-title = Логи поддержки и удалённое управление
admin-support-panel-subtitle = Клиентские логи, дампы падений и удалённые действия
admin-support-close-game = Закрыть игру
admin-support-restart-launcher = Перезапустить лаунчер
admin-support-delivered-bundles = Полученные дампы ({ $count })
admin-support-nobundles-player = Игрок ещё не отправлял дампы поддержки.
admin-support-req-history = История запросов ({ $count })
admin-support-norequests-player = Запросы логов игроку ещё не отправлялись.

## Пользователи и детальный профиль
admin-users-title = Пользователи
admin-users-subtitle = Профили, блокировки, роли и персональные права.
admin-users-player = Игрок
admin-users-discord = Discord
admin-users-roles = Роли
admin-users-status = Статус
admin-users-banned = заблокирован
admin-users-active = активен
admin-users-empty-title = Пользователей пока нет
admin-users-tab-profile = Профиль и права
admin-users-tab-skins = Скины и плащи
admin-users-tab-support = Поддержка и логи
admin-users-tab-mod = Модерация
admin-users-not-found = Пользователь не найден
admin-users-direct-perms = Персональные права
admin-users-direct-perms-hint = Назначаются игроку поверх ролей. Выберите сборки, для которых действует право, или примените ко всем.
admin-users-skin-preview = 3D Превью скина
admin-users-custom-skin = Пользовательский скин
admin-users-uploaded = Загружен
admin-users-default-skin = По умолчанию
admin-users-upload-skin = Загрузить скин
admin-users-reset-skin = Сбросить скин
admin-users-presets-title = Галерея скинов
admin-users-presets-subtitle = Сохранённые пресеты скинов игрока
admin-users-active-cape = Активный плащ
admin-users-active-cape-subtitle = Выберите, какой из выданных плащей надеть на модель игрока
admin-users-no-cape = Без плаща (Отключён)
admin-users-granted-capes = Выданные плащи
admin-users-granted-capes-subtitle = Переключайте плащи из каталога сервера, доступные игроку для выбора в кабинете и лаунчере

## Список серверов в админке
admin-servers-title = Сервера
admin-servers-subtitle = Сборки, порядок отображения и метаданные запуска
admin-servers-new = Новый сервер
admin-servers-col-stack = Стек
admin-servers-col-status = Статус
admin-servers-empty-title = Серверов пока нет
admin-servers-empty-text = Создайте первый профиль сервера для начала работы.
admin-servers-create-title = НОВЫЙ СЕРВЕР
admin-servers-create-subtitle = Создание сервера и его первой сборки
admin-servers-name = Название сервера
admin-servers-name-hint = Адреса указываются в каждом игровом сервере после создания сборки.
admin-servers-initial-build = Начальная конфигурация сборки
admin-servers-build-version = Версия сборки
admin-servers-create-later = Вы сможете создать сборку позже в настройках сервера.
admin-servers-create-btn = Создать сервер

## Права, роли, диагностика и плащи
admin-roles-select = Выберите роль
admin-perm-context = Контекст
admin-perm-all-builds = Все сборки
admin-perm-permission = Право (node)
admin-perm-add = Добавить
admin-perm-suggestions-unavailable = Автодополнение недоступно: { $error }. Права можно ввести вручную.
admin-perm-already-granted = Право уже выдано во всех выбранных контекстах.
admin-perm-pick-build = Выберите хотя бы одну сборку или выдайте право для всех сборок.
admin-perm-no-permissions = Персональные права ещё не выданы.
admin-diag-title = Диагностика
admin-diag-collect = Запросить
admin-diag-subtitle = Версии, железо и скорость соединения. Никаких личных данных.
admin-diag-empty = Данные ещё не собирались.
admin-diag-verify = Проверить файлы
admin-diag-clear-assets = Очистить кэш
admin-diag-restart-launcher = Перезапустить лаунчер
admin-capes-equip = Надеть
admin-capes-access-granted = Доступ разрешён
admin-capes-access-not-granted = Доступ не разрешён
admin-capes-selected = Выбран
admin-capes-granted = Выдан
admin-capes-locked = Заблокирован
admin-capes-count-granted = { $count } из { $total } выдано
admin-capes-presets-count = { $count } пресетов
admin-capes-empty-presets = У игрока ещё нет сохранённых пресетов.

## Админка: Главный дашборд
admin-dash-title = Администрирование
admin-dash-subtitle = Панель управления главным сервером
admin-dash-card-users = Пользователи
admin-dash-card-servers = Сервера
admin-dash-card-builds = Сборки
admin-dash-card-online = Лаунчеры онлайн
admin-dash-data-state = Состояние данных
admin-dash-filestore = Хранилище файлов
admin-dash-backup-btn = Скачать бэкап БД
admin-dash-backup-hint = Архив pg_restore базы данных мастера — аккаунты, права, скины и плащи.
admin-dash-quick-actions = Быстрые действия
admin-dash-create-client = Создать клиент
admin-dash-publish-news = Опубликовать новость
admin-dash-deploy-launcher = Развернуть лаунчер

## Админка: Вкладки раздела сборки
admin-tab-profile = Профиль
admin-tab-profile-hint = Название, порядок и ресурсы
admin-tab-mods = Моды
admin-tab-mods-hint = Установленные моды и каталог Modrinth
admin-tab-build = Сборка и файлы
admin-tab-build-hint = Импорт сборки, файлы и публикация
admin-tab-instances = Игровые сервера
admin-tab-instances-hint = Сервера и обёртки
admin-tab-client = Клиент по умолчанию
admin-tab-client-hint = RAM, флаги JVM и опциональные моды

## Админка: Селектор сборок
admin-build-active = Активная сборка
admin-build-total = всего { $count }
admin-build-subtitle = Выберите версию сборки для настройки файлов, импорта и публикации.
admin-build-live = релиз
admin-build-draft = черновик
admin-build-new = Новая сборка

## Админка: Импорт паков
admin-import-title = Импорт сборки
admin-import-subtitle = Modrinth и CurseForge
admin-import-format = Формат сборки
admin-import-drop = Нажмите или перетащите файл сборки
admin-import-process = Обработать сборку

## Админка: Ручная загрузка файлов
admin-manual-title = Ручная загрузка
admin-manual-subtitle = Прямая инъекция артефактов
admin-manual-drop = Перетащите файл сюда
admin-manual-path = Путь назначения
admin-manual-add = Добавить в сборку

## Админка: Файловый менеджер
admin-fm-title = Файловый менеджер
admin-fm-subtitle = Обозреватель файлов и правила синхронизации
admin-fm-open = Открыть
admin-fm-edit = Редактировать
admin-fm-download = Скачать
admin-fm-rename = Переименовать
admin-fm-copy-path = Скопировать путь
admin-fm-sync-mode = Режим синхронизации
admin-fm-new-folder = Новая папка
admin-fm-upload = Загрузить файлы
admin-fm-delete = Удалить
admin-fm-col-name = Имя
admin-fm-col-sync = Синхр.
admin-fm-col-size = Размер
admin-fm-col-kind = Тип
admin-fm-items = элементов
admin-fm-folder = Папка
admin-fm-empty = Пустая папка
admin-fm-sync-synced = Синхронизируется — версия сервера всегда главная
admin-fm-sync-ignored = Игнорируется — никогда не скачивается и не удаляется
admin-fm-sync-user = Пользовательский — устанавливается один раз

## Админка: Создание сборки
admin-modal-new-build = НОВАЯ СБОРКА
admin-modal-new-build-sub = Версия, Minecraft и параметры загрузчика
admin-modal-copy-from = Копировать из
admin-modal-start-empty = Начать с пустой
admin-modal-copy-hint = Переносит все файлы и настройки. Повторная загрузка не требуется.
admin-modal-build-version = Версия сборки
admin-modal-optional-vanilla = Необязательно для vanilla

## Каталог модов
admin-mods-title = КАТАЛОГ МОДОВ
admin-mods-subtitle = Modrinth и CurseForge в одном месте
admin-mods-search-placeholder = Искать моды…  ( / )
admin-mods-sort-relevance = По релевантности
admin-mods-sort-downloads = По скачиваниям
admin-mods-sort-follows = По подписчикам
admin-mods-sort-updated = Недавно обновлённые
admin-mods-sort-newest = Новейшие
admin-mods-results-count = { $count } результат(ов)
admin-mods-results-none = Результатов не найдено
admin-mods-page = Страница
admin-mods-source = Исходники
admin-mods-issues = Баги
admin-mods-tab-versions = Версии
admin-mods-tab-about = Описание
admin-mods-tab-gallery = Галерея
admin-mods-matching-only = Только версии для этой сборки
admin-mods-no-versions = Нет подходящих версий.
admin-mods-no-screenshots = Скриншотов нет.
admin-mods-downloads-count = скачиваний

## Плащи
admin-capes-title = ПЛАЩИ
admin-capes-subtitle = Каталог плащей и управление косметикой
admin-capes-add-title = Добавить новый плащ
admin-capes-add-subtitle = Загружайте текстуры плащей Minecraft 64x32 или 22x17 PNG.
admin-capes-name-label = Название плаща
admin-capes-name-placeholder = например, Mojang 2011, Вишня...
admin-capes-dropzone = Перетащите PNG файл сюда или нажмите для выбора
admin-capes-dropzone-hint = Текстура PNG до 512 КБ
admin-capes-upload-btn = Загрузить плащ
admin-capes-uploading = Загрузка…
admin-capes-grid-title = Сетка каталога плащей
admin-capes-empty-title = В каталоге нет плащей
admin-capes-empty-text = Загружайте PNG-текстуры плащей, чтобы игроки могли надевать их.

## Целостность
admin-integrity-title = ЦЕЛОСТНОСТЬ
admin-integrity-subtitle = Что лаунчеры обнаружили перед запуском игры
admin-integrity-note = Сигнал на стороне клиента, а не доказательство: лаунчер имеет открытый исходный код, и пропатченная сборка сообщает всё что угодно. Относитесь к этому как к поводу проверить, но не к автоматическому бану.
admin-integrity-unreviewed-only = Только непроверенные
admin-integrity-col-when = Когда
admin-integrity-col-finding = Находка
admin-integrity-col-subject = Предмет
admin-integrity-col-build = Сборка
admin-integrity-col-player = Игрок
admin-integrity-repaired = исправлено
admin-integrity-launcher = лаунчер
admin-integrity-open-card = открыть карточку
admin-integrity-reviewed = проверено
admin-integrity-empty-title = Нарушений не зафиксировано
admin-integrity-empty-text = Лаунчеры проверяют моды и конфиги по подписанному манифесту перед каждым запуском.

## Блокировки
admin-blocklist-mask-placeholder = *xray*
admin-blocklist-sha1-placeholder = 40 hex символов
admin-blocklist-reason-placeholder = Известный чит-пак

## Правила
admin-rules-title = ПРАВИЛА
admin-rules-subtitle = Свод правил, который читают игроки и цитируют модераторы
admin-rules-search-placeholder = Поиск по коду, названию или формулировке
admin-rules-scope = Область
admin-rules-scope-all = Все области
admin-rules-public-page = Публичная страница
admin-rules-btn-section = Раздел
admin-rules-btn-rule = Правило
admin-rules-empty-title = Правил пока нет
admin-rules-empty-text = Начните с раздела, например «1 · Игровой процесс», затем добавьте правила внутрь.
admin-rules-other-section = Другие правила
admin-rules-outside-section = вне разделов
admin-rule-title-placeholder = Гриферство построек других игроков
admin-rule-text-placeholder = Что именно считается нарушением данного правила, а что нет
admin-sanc-min-placeholder = 30м · пусто = без минимума
admin-sanc-max-placeholder = 7д · пусто = бессрочно
admin-sanc-label-placeholder = первое нарушение · повторное · в грубой форме
admin-rule-delete-confirm = Удалить правило { $code } «{ $title }»? Выданные наказания сохранят свой текст.
admin-rule-delete-fail = Не удалось удалить правило
admin-rule-save-fail = Не удалось сохранить правило
admin-cat-delete-confirm = Удалить раздел «{ $name }»? Находящиеся в нём правила переместятся в «Другие правила».
admin-cat-delete-fail = Не удалось удалить раздел
admin-cat-save-fail = Не удалось сохранить раздел

## Wrapper
admin-wrapper-download-btn = Скачать wrapper.jar
admin-wrapper-subtitle = Все параметры, которые читает wrapper.
admin-wrapper-key = КЛЮЧ
admin-wrapper-default = ПО УМОЛЧАНИЮ
admin-wrapper-meaning = ЗНАЧЕНИЕ
admin-wrapper-required = обязательно

## Аудит
admin-audit-title = АУДИТ
admin-audit-subtitle = Кто, что и когда изменил
admin-audit-event = Событие
admin-audit-all-events = Все события
admin-audit-target = Цель
admin-audit-anything = Любая
admin-audit-target-id = ID цели
admin-audit-optional = опционально
admin-audit-uuid-placeholder = UUID
admin-audit-apply = Применить
admin-audit-reset = Сбросить
admin-audit-load-more = Загрузить ещё
admin-audit-empty-title = Записей пока нет
admin-audit-empty-text = Входы, запуски, проверки целостности и действия админов попадают сюда в реальном времени.

## Настройки
admin-settings-restart-title = Требуется перезапуск
admin-settings-restart-desc = Настройки считываются при старте. Изменения вступят в силу после перезапуска мастера.
admin-settings-sec-general = Общие
admin-settings-sec-auth = Авторизация
admin-settings-sec-storage = Хранилище
admin-settings-sec-integrations = Интеграции

## Обёртка сервера (ServerWrapper)
admin-wrapper-title = ОБЁРТКА СЕРВЕРА
admin-wrapper-lead = Инсталлятор агентов и супервизор для игровых серверов
admin-wrapper-setup-intro = ServerWrapper устанавливает нужный агент для вашего сервера, подготавливает authlib-injector и сохраняет сервер видимым в лаунчере во время загрузки. Это инсталлятор и супервизор — разграничение прав остаётся на мастере.
admin-wrapper-setup-not-built = Ещё не собрано — запустите ./gradlew collectAgents в папке agent/ и скопируйте agent/build/agents/ в {NORO_DATA_DIR}/agents/.
admin-wrapper-setup-step2-desc = Секрет агента генерируется для каждого игрового сервера в админке сборки, в разделе Игровые сервера. Он хранится только здесь — wrapper передаёт его процессу сервера через переменную окружения.
admin-wrapper-setup-step2-min = Это минимум. NeoForge и Forge запускаются из файла аргументов, а не из jar — передавайте его с префиксом @, например server-jar=@libraries/net/neoforged/neoforge/21.1.248/unix_args.txt, и сохраните их файл JVM как jvm-args=@user_jvm_args.txt. Все остальные опции описаны ниже.
admin-wrapper-setup-step3-desc = Wrapper определяет платформу и версию Minecraft, устанавливает подходящий агент, проверяет его подпись и запускает сервер.
admin-wrapper-setup-step4-desc = Сессии проверяются на этом мастере, поэтому сервер должен оставаться в режиме online-mode=true.

admin-wrapper-opt-master-url = Адрес главного узла master. Завершающий слэш отсекается.
admin-wrapper-opt-secret = Секрет агента для этого игрового сервера, выданный в админке сборок. Должен начинаться с noroagent_. Хранится только здесь — wrapper передаёт его процессу сервера через переменную окружения.
admin-wrapper-opt-server-jar = Jar-файл сервера относительно server-dir. Начинается с @ для файла аргументов — NeoForge и Forge запускаются именно так: @libraries/net/neoforged/neoforge/21.1.248/unix_args.txt
admin-wrapper-opt-signing-public-key = Ключ ed25519 мастера, hex. Пустое значение означает получение один раз и привязку к noro/signing-key.pub; последующее изменение вызовет ошибку. В продакшене задавайте явно.
admin-wrapper-opt-server-dir = Рабочая директория сервера. Все остальные пути рассчитываются относительно неё, агент помещается в plugins/ или mods/.
admin-wrapper-opt-java = Исполняемый файл Java. Укажите конкретную JDK, если стандартная не подходит для данной версии Minecraft.
admin-wrapper-opt-jvm-args = Аргументы JVM через пробел. Принимает @-файл: NeoForge хранит свои как @user_jvm_args.txt.
admin-wrapper-opt-server-args = Аргументы, передаваемые после jar или файла аргументов. Оставьте пустым, если передавать нечего.
admin-wrapper-opt-platform = paper, fabric, neoforge или forge. Переопределяет автоопределение — нужно, когда в libraries/ лежат несколько версий.
admin-wrapper-opt-mc-version = Версия Minecraft, например 1.21.1. Переопределяет автоопределение.
admin-wrapper-fb-fetch-pin = получить и привязать
admin-wrapper-fb-detected = определяется

admin-agent-title = Агенты
admin-agent-lead = Wrapper устанавливает их автоматически — скачивайте вручную только для ручной установки.
admin-agent-not-built = Ничего ещё не собрано. Запустите ./gradlew collectAgents в папке agent/ и скопируйте agent/build/agents/ в {NORO_DATA_DIR}/agents/.
admin-agent-versions-count = { $count } версий

































