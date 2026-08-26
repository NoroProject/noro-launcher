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
sidebar-no-identity = Платформа не привязана

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
profile-skin-model = Модель
profile-skin-model-classic = Классическая
profile-skin-model-slim = Тонкая
notif-skin-model-failed = Не удалось сменить модель скина: { $reason }
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
notif-rate-limited = Слишком много попыток — повторите через минуту
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
login-subtitle = ВХОД
login-sign-in = ВОЙТИ
login-waiting = ОЖИДАНИЕ...
login-checking = ПРОВЕРКА СЕССИИ...
login-save-session = Запомнить сессию
login-auto-login = Входить автоматически
login-tagline = Лаунчер Minecraft серверов с автоматической установкой модов и скинов
login-sign-in-web = Войти
login-web-hint = Откроется сайт — там можно войти любой привязанной платформой

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
mods-limited = ПО ПРАВУ
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
web-home-lead = Модовый проект Minecraft с удобным лаунчером: авто-скачивание модов, вход в один клик, кастомные скины и плащи — всё в одном месте.
web-home-cta-cabinet = ОТКРЫТЬ КАБИНЕТ
web-home-cta-sign-in = НАЧАТЬ
web-home-stat-online = В сети
web-home-stat-servers = Серверов
web-home-worlds-eyebrow = Миры
web-home-worlds-title = Серверы проекта
web-home-worlds-all = Все серверы
web-home-feature-builds-title = Быстрая установка модов
web-home-feature-builds-text = Лаунчер сам загрузит и обновит все нужные моды и файлы — вам остаётся только нажать «Играть».
web-home-feature-identity-title = Единый профиль
web-home-feature-identity-text = Вход в один клик через Telegram, Discord, Twitch или Google, скин, плащ и личный кабинет — всё в одном аккаунте.
web-home-feature-rules-title = Честная игра
web-home-feature-rules-text = Прозрачные правила и открытая модерация: правила всегда под рукой и понятны каждому.
web-home-flow-eyebrow = Путь до игры
web-home-flow-title = От входа до Minecraft за четыре шага
web-home-step-signin-title = Авторизуйтесь
web-home-step-signin-text = Войдите на сайте любой доступной платформой — аккаунт сразу готов к игре.
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
web-download-lead = Войдите, выберите сервер — остальное лаунчер синхронизирует сам.
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
cabinet-apps-title = Приложения
cabinet-apps-subtitle = Управление приложениями с доступом к аккаунту
cabinet-apps-lead = Сторонние сервисы и лаунчеры, у которых есть доступ к вашему профилю
cabinet-apps-refresh = Обновить
cabinet-apps-default-desc = Доступ к вашему профилю Noro Network
cabinet-apps-revoke = Отозвать доступ
cabinet-apps-none-title = У вас нет подключённых сторонних приложений
cabinet-apps-none-text = Здесь будут отображаться приложения и лаунчеры, которым вы разрешили доступ

## OAuth2 Авторизация
oauth-loading-app = Загрузка информации о приложении…
oauth-official-app = Официальное приложение
oauth-requested-permissions = Что получит приложение
oauth-wants-access = запрашивает доступ к аккаунту { $user }
oauth-by-developer = разработчик — { $owner }
oauth-app-pending = Приложение ещё не проверено, поэтому войти в него может только автор.
oauth-will-return-to = Вернёт на { $host }
oauth-revoke-hint = Доступ можно отозвать в любой момент в личном кабинете.
oauth-cannot-continue = Не получится продолжить
oauth-back-home = Вернуться на сайт
oauth-missing-params = Приложение прислало неполный запрос: нужны client_id и redirect_uri.
oauth-scope-identity = Ник, UUID и аватар
oauth-scope-profile = Роли и состояние аккаунта
oauth-scope-skins = Скин и плащ
oauth-scope-capes = Доступные плащи
oauth-scope-punishments = Действующие наказания
oauth-scope-servers = Список серверов проекта и онлайн
oauth-scope-skins-write = Менять скин и плащ
oauth-scope-identities = Видеть, чем вы входите
oauth-scope-journal = Историю запусков: сборки и моды
oauth-scope-launcher = Полный доступ к аккаунту
oauth-deny = Отмена
oauth-allow = Разрешить

## Менеджер скинов и плащей
skin-title = Скины и плащи
skin-subtitle = Менеджер скинов в стиле Modrinth & Pandora
skin-3d-character = 3D Персонаж
skin-custom-badge = Свой скин
skin-default-badge = По умолчанию
skin-reset-default = Сбросить на стандартный
skin-model = Модель
skin-model-classic = Классическая
skin-model-slim = Тонкая
skin-model-hint = У тонкой модели руки на пиксель уже — выберите ту, под которую нарисован скин
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
login-with = ВОЙТИ ЧЕРЕЗ { $provider }
login-no-methods = На этом инстансе ещё не настроен ни один способ входа. Оператор должен включить его в админке → Вход.
login-passkey = ВОЙТИ ЧЕРЕЗ PASSKEY
login-recovery-toggle = Использовать код восстановления
login-username-placeholder = Имя пользователя
login-submit = ВОЙТИ
login-recovery-hint = Каждый код работает один раз.

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
admin-rule-punish-reason = Причина в наказании
admin-rule-punish-reason-placeholder = Оскорбление участников
admin-rule-punish-reason-hint = Что прочитает наказанный, когда модератор назвал этот пункт и ничего не написал. Заголовок правила тут не годится: «Уважение к другим игрокам» читается как похвала, а не как причина.
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
admin-set-label-hero_image_url = URL главной иллюстрации (Hero Render)
admin-set-hint-hero_image_url = Иллюстрация персонажа на главной странице сайта. Меняется мгновенно без перезапуска.
admin-settings-hero-title = Главная иллюстрация (Hero Render)
admin-settings-hero-desc = Картинка персонажа на главной странице. Меняется мгновенно при загрузке файла без перезапуска сервера.
admin-settings-hero-upload = Загрузить картинку
admin-set-hint-allowed_origins = Через запятую. Пустое значение означает доступ для любого origin.
admin-set-label-login_image_url = Адрес картинки страницы входа
admin-set-hint-login_image_url = Подставляется сам, когда загружаешь файл.
admin-set-hint-instance_name = Показывается в шапке сайта и заголовком главной страницы.
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
admin-role-icon-hint = Один символ у ника.
admin-role-prefix-label = Префикс в игре
admin-role-prefix-hint = Перед ником. Цвета через &.
admin-role-badge-preview = Плашка в игре
admin-role-badge-hint = Видят принявшие ресурспак.
admin-role-badge-failed = Не удалось нарисовать плашку
admin-role-badge-own = Своя картинка
admin-role-badge-pick = Загрузить
admin-role-badge-clear = Убрать
admin-role-badge-uploaded = Картинка загружена
admin-role-badge-cleared = Вернули нарисованную плашку
admin-role-badge-rules = PNG, высота кратна 7 (до 56), ширина до 12 высот. Заменяет плашку целиком.
admin-role-section-look = Как выглядит
admin-role-section-game = В игре
admin-role-section-rights = Права и связи
admin-role-saved = Роль сохранена
admin-role-saved-hint = Плашка обновится после синхронизации в списке ролей.
admin-roles-sync = Синхронизировать
admin-roles-synced = Плашки пересобраны
admin-roles-sync-hint = Пересобрать плашки и разослать их игрокам.
admin-role-suffix-label = Суффикс в игре
admin-role-suffix-hint = После ника.
admin-role-inherits-label = Наследует от
admin-role-inherits-none = Ничего — только собственные права
admin-role-inherits-hint = Права родителя работают здесь.
admin-role-lp-label = Группа LuckPerms
admin-role-lp-hint = Игровая группа.
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
admin-tokens-perms-label = Права доступа
admin-tokens-preset-title = Готовые шаблоны ролей:
admin-tokens-preset-superadmin = Полный доступ
admin-tokens-preset-fulladmin = Админ панели
admin-tokens-preset-senior-mod = Модератор
admin-tokens-preset-junior-mod = Хелпер
admin-tokens-preset-custom = Свой набор
admin-tokens-custom-toggle = Ввести права вручную текстом
admin-tokens-checkbox-toggle = Выбор галочками
admin-tokens-select-all = Выбрать всё
admin-tokens-deselect-all = Снять всё

## Модерация пользователей
admin-users-version = Версия
admin-users-login-as = Войти от имени
admin-users-req-logs = Запросить логи
admin-users-end-sessions = Завершить все сессии
admin-users-impersonate-title = Войти от имени игрока
admin-users-impersonate-warn = Игрок не уведомляется. Все действия, совершённые вами в его профиле, фиксируются в журнале аудита под вашим именем.
admin-users-impersonate-code = Код восстановления
admin-users-impersonate-stepup = Подтвердите, что это вы. Подтверждение действует { $mins } минут.
admin-users-impersonate-passkey = Подтвердить ключом доступа
admin-users-impersonate-use-code = Нет ключа доступа? Используйте код восстановления
admin-users-impersonate-use-passkey = Подтвердить ключом доступа
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
nav-admin-automod = Фильтры чата
nav-admin-moderation = Тексты наказаний
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
admin-gs-maintenance = Техработы
admin-gs-maintenance-reason = Причина техработ (опционально)
admin-gs-maintenance-enable-all = Включить техработы для всех
admin-gs-maintenance-disable-all = Выключить техработы для всех
admin-gs-maintenance-countdown = Время предупреждения перед киком
admin-gs-maintenance-countdown-imm = Немедленно (0 сек.)
admin-gs-maintenance-countdown-30s = 30 секунд
admin-gs-maintenance-countdown-60s = 1 минута
admin-gs-maintenance-countdown-300s = 5 минут
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
admin-news-search-placeholder = Поиск по заголовку или тексту
paging-range = { $from }–{ $to } из { $total }
paging-page = страница { $page } из { $pages }
paging-empty = По запросу ничего не нашлось
admin-users-subtitle = Профили, блокировки, роли и персональные права.
admin-users-player = Игрок
admin-users-identity = Платформа
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
admin-users-identities = Способы входа
admin-users-identities-hint = Платформы, которыми игрок заходит. Отвяжите, если он потерял к ней доступ.
admin-users-identities-none = Привязок нет
admin-users-identities-none-hint = Локальный аккаунт, заведённый персоналом: платформы за ним нет.
admin-users-identity-primary-hint = Через эту платформу аккаунт и создан — из неё выведен MC-UUID, отвязать нельзя.
admin-users-account-id = ID аккаунта
admin-users-registered = Зарегистрирован
admin-users-last-login = Последний вход
admin-users-ban-reason = Причина бана
admin-users-flag-root = root
admin-users-flag-local = локальный аккаунт
admin-users-flag-no-play = не может играть
admin-users-flag-hidden = скрыт из онлайна
admin-users-flag-silent = тихий вход
admin-users-flag-frozen = заморожен
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
admin-perm-no-permissions = Персональные права ещё не выданы.
admin-perm-show-nodes = Показать все узлы
admin-perm-hide-nodes = Скрыть категории
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
# Счётчик рядом с числом: «3 правила синхронизации».
admin-sync-rules-btn = правила синхронизации
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
admin-settings-tab-branding = Брендинг
admin-settings-tab-general = Общие
admin-settings-tab-sign-in = Вход
admin-settings-tab-storage = Хранилище
admin-settings-tab-integrations = Интеграции
admin-settings-tab-health = Состояние
admin-settings-logo-title = Логотип
admin-settings-logo-desc = Значок в шапке сайта. Пусто — берётся значок из сборки.
admin-settings-image-transparent = С прозрачностью — на сайте без рамки
admin-settings-image-opaque = Без прозрачности — на сайте в рамке
admin-set-label-logo_url = Адрес логотипа
admin-set-hint-logo_url = Подставляется сам, когда загружаешь файл.

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


































## Тексты наказаний
admin-moderation-title = Тексты наказаний
admin-moderation-subtitle = Что видит игрок при бане, муте и предупреждении
admin-moderation-reset = Вернуть встроенный
admin-moderation-vars-title = Подстановки
admin-moderation-vars-lead = Всё остальное остаётся в тексте как есть. Цвета пишутся через &, например &c.
admin-moderation-var-player = Наказанный игрок
admin-moderation-var-reason = Причина, которую указал модератор
admin-moderation-var-duration = Сколько осталось, например 6d 23h
admin-moderation-var-expires = Дата окончания, UTC
admin-moderation-var-actor = Кто выдал
admin-moderation-var-rule = Код правила либо прочерк
admin-moderation-var-id = Номер дела, первые восемь знаков
admin-moderation-var-kind = banned, muted или warned
admin-moderation-ban-perm = Бан сети, навсегда
admin-moderation-ban-perm-hint = Экран отключения. Это весь разговор с игроком — напишите, куда подавать апелляцию.
admin-moderation-ban-temp = Бан сети, со сроком
admin-moderation-ban-temp-hint = Тот же экран, но игрок должен ещё узнать, когда бан кончится.
admin-moderation-sban-perm = Бан сервера, навсегда
admin-moderation-sban-perm-hint = Показывается, когда закрыт только этот сервер.
admin-moderation-sban-temp = Бан сервера, со сроком
admin-moderation-sban-temp-hint = На остальных серверах сети игрок продолжает играть.
admin-moderation-mute-perm = Мут, навсегда
admin-moderation-mute-perm-hint = Ответ на каждое сообщение, которое пытается написать замученный.
admin-moderation-mute-temp = Мут, со сроком
admin-moderation-mute-temp-hint = То же, но с остатком срока. Молчание в ответ читается как поломка сервера.
admin-moderation-warn = Предупреждение
admin-moderation-warn-hint = Показывается сразу, если игрок в сети, иначе — при следующем входе.
admin-moderation-broadcast = Объявление
admin-moderation-broadcast-hint = Видят все на сервере. Пусто — наказывать молча.
admin-moderation-receipt = Подтверждение модератору
admin-moderation-receipt-hint = Что видит в ответ тот, кто выдал наказание.
admin-moderation-mute-bar-perm = Мут над хотбаром, навсегда
admin-moderation-mute-bar-temp = Мут над хотбаром, со сроком
admin-moderation-mute-bar-hint = Одна строка без переносов: всё длиннее actionbar обрезает.
admin-moderation-warn-bar = Предупреждение над хотбаром
admin-moderation-reason = Причина по правилу
admin-moderation-reason-hint = Берётся, когда модератор назвал правило и ничего не написал. Своя формулировка у пункта важнее этого шаблона.
admin-moderation-no-account = Экран «нет аккаунта»
admin-moderation-no-account-hint = Видит тот, кого мастер не знает, — настоящая граница доступа, до сервера можно дойти и мимо лаунчера
admin-moderation-no-access = Экран «нет доступа»
admin-moderation-no-access-hint = Видит тот, у кого аккаунт есть, а доступа к этой сборке нет
admin-moderation-maintenance = Экран техработ
admin-moderation-maintenance-hint = Видят все, кроме обладателей noro.server.maintenance.bypass
admin-moderation-var-rule-title = Название правила
admin-moderation-var-rule-link = Кликабельный код правила

## Опциональные моды: отказ при включении
optional-conflicts-with = Не совместим с модом { $mod }. Выключите его, если хотите включить этот.
optional-needs-first = Сначала включите { $mod }: без него этот мод не работает.

## Жалобы и заморозка
admin-reports-title = Жалобы игроков
admin-reports-subtitle = Очередь репортов с игровых серверов
admin-reports-open-only = Только открытые
admin-reports-resolve = Закрыть репорт
admin-reports-empty-title = Жалоб нет
admin-reports-empty-text = В данный момент открытых жалоб от игроков нет.
admin-freezes-section-title = Заморозка / Разморозка игрока
admin-freezes-reason-label = Причина заморозки
admin-freezes-reason-placeholder = Проверка на софт / подозрение в читах
admin-freezes-action-freeze = Заморозить
admin-freezes-action-unfreeze = Разморозить
admin-freezes-frozen = Заморожен
admin-freezes-active = Активен
nav-admin-reports = Жалобы
cabinet-privacy-title = Приватность в онлайне
cabinet-privacy-hide-online = Скрыть меня из публичного списка онлайна
cabinet-privacy-silent-join-title = Войти тихо (Vanish на входе)
cabinet-privacy-silent-join-hint = Автоматически входить на сервер в режиме невидимости
cabinet-profile-title = Профиль
cabinet-roles-card = Роли
cabinet-direct-card = Прямые права
cabinet-activity-title = Игровая активность (Heatmap)
cabinet-activity-days = { $count } дней
cabinet-activity-minutes = { $count } мин
cabinet-activity-hours = { $count } ч
cabinet-activity-hour-short = ч
cabinet-activity-total = за год
cabinet-activity-best = лучший день
cabinet-activity-none = не играл
cabinet-activity-less = меньше
cabinet-activity-more = больше
cabinet-activity-empty-title = Нет данных об активности
cabinet-activity-empty-text = Сыгранные часы отобразятся после первых игровых сессий.

## Вкладка действий и внутриигровые команды
admin-users-tab-game-actions = Игровые действия
admin-game-actions-title = Прямые игровые действия
admin-game-actions-subtitle = Отправка команд на активный игровой сервер для @{ $username }
admin-game-actions-status-online = Онлайн на { $server }
admin-game-actions-status-offline = Офлайн
admin-game-actions-status-unknown = Статус неизвестен — мастер не ответил
admin-game-actions-offline-title = Игрок вне игры
admin-game-actions-offline-text = Игровые действия недоступны, так как игрок @{ $username } сейчас не подключён ни к одному серверу.
admin-game-actions-kick-title = Кикнуть @{ $username }
admin-game-actions-kick-desc = Принудительно отключить игрока от активного сервера.
admin-game-actions-kick-reason = Причина кика
admin-game-actions-kick-placeholder = Кикнут администрацией
admin-game-actions-kick-btn = Кикнуть игрока
admin-game-actions-tell-title = Личное сообщение (/tell)
admin-game-actions-tell-desc = Отправить приватное чат-сообщение игроку @{ $username }.
admin-game-actions-tell-msg = Текст сообщения
admin-game-actions-tell-placeholder = Проверьте дискорд / тикет поддержки
admin-game-actions-tell-btn = Отправить сообщение

## Объявления на серверах
admin-gs-announce-title = Объявление на игровые серверы
admin-gs-announce-target = Целевой сервер
admin-gs-announce-all = Все серверы сборки
admin-gs-announce-send = Отправить объявление
admin-gs-announce-placeholder = Введите текст объявления для игроков...

## Предложенные моды
admin-mod-suggestions-title = Предложенные моды от игроков ({ $count })
admin-mod-suggestions-expand = Посмотреть списком в окне
admin-mod-suggestions-search = Поиск по названию, автору или описанию...
admin-mod-suggestions-open-external = Перейти на страницу мода
admin-mod-suggestions-accept = Принять
admin-mod-suggestions-accept-optional = Добавить как опциональный мод
admin-mod-suggestions-accept-regular = Добавить как обычный мод
admin-mod-suggestions-accept-servers = Обычный + установить на серверы
admin-mod-suggestions-reject = Отклонить
admin-mod-suggestions-empty = Нет предложенных модов по запросу

## Установленные моды сборки
admin-installed-mods-title = Моды сборки
admin-installed-mods-found = Найдено { $count } из { $total } модов
admin-installed-mods-count = Установлено модов в сборке: { $count }
admin-installed-mods-search = Поиск модов...
admin-installed-mods-grid = Сетка
admin-installed-mods-list = Список
admin-installed-mods-upload = Загрузить JAR
admin-installed-mods-catalog = Каталог модов
admin-installed-mods-no-build = Выберите или создайте версию сборки выше, чтобы управлять установленными модами.
admin-installed-mods-empty-title = Моды еще не установлены
admin-installed-mods-empty-text = Откройте каталог Modrinth для выбора модов или перетащите файл .jar сюда.
admin-installed-mods-browse = Открыть каталог Modrinth
admin-installed-mods-no-match = Ничего не найдено по запросу "{ $query }"




## Быстрый поиск (Spotlight)
spotlight-quick-search = Быстрый поиск...
spotlight-placeholder = Введите команду, страницу, ник или сервер... (Ctrl+K)
spotlight-no-results = Ничего не найдено по запросу "{ $query }"
spotlight-nav-instructions = Перемещение с помощью ↑ ↓
spotlight-open-instructions = Открыть через Cmd+K / Ctrl+K
spotlight-prefixes = @ люди · # серверы · > команды
spotlight-group-nav = Разделы
spotlight-group-users = Игроки
spotlight-group-servers = Серверы
spotlight-group-actions = Команды
spotlight-action-locale = Сменить язык на { $locale }
spotlight-action-logout = Выйти

## Поиск и фильтры списка пользователей
admin-users-search-label = Поиск игрока / UUID / платформа
admin-users-search-placeholder = Поиск по нику, платформе или UUID...
admin-users-status-label = Статус
admin-users-status-all = Все статусы
admin-users-status-active-only = Только активные
admin-users-status-banned-only = Только забаненные
admin-users-role-label = Роль
admin-users-role-all = Все роли



## Фильтры чата (AutoMod)
admin-automod-title = Фильтры чата
admin-automod-subtitle = Что чат ловит сам и что за этим следует
admin-automod-load-failed = Не удалось загрузить фильтры чата
admin-automod-enabled = ВКЛЮЧЁН
admin-automod-disabled = ВЫКЛЮЧЕН
admin-automod-mode = Действие
admin-automod-mode-deny = Не пропускать — сообщение не уйдёт в чат
admin-automod-mode-escalate = С предупреждением — сначала предупредить, при повторе замутить
admin-automod-mode-punish = Наказывать — мут сразу
admin-automod-mode-shadow = Тихий режим — пропускать, но сообщать модерации
admin-automod-rule-code = Пункт свода
admin-automod-rule-code-hint = 1.4 или 2.2
admin-automod-whitelist = Разрешённые домены, через запятую
admin-automod-words = Запрещённые слова, через запятую
admin-automod-words-hint = слово, ещё слово
admin-automod-threshold = Доля капса, от 0.1 до 1.0
admin-automod-min-length = Минимальная длина сообщения
admin-automod-max-messages = Сколько сообщений можно
admin-automod-window = За сколько секунд
admin-automod-ad-title = Реклама
admin-automod-ad-hint = Ссылки, домены и IP-адреса в чате
admin-automod-word-title = Слова
admin-automod-word-hint = Запрещённые слова и мат
admin-automod-caps-title = Капс
admin-automod-caps-hint = Сообщения, набранные ЗАГЛАВНЫМИ
admin-automod-flood-title = Флуд
admin-automod-flood-hint = Повторяющиеся сообщения подряд
admin-automod-empty-title = Фильтров нет
admin-automod-empty-text = Фильтры чата ещё не настроены.

## Дела: разбор жалоб
nav-admin-cases = Дела
admin-cases-title = Дела
admin-cases-subtitle = Жалобы игроков, собранные в один разбор
admin-cases-open-only = Только открытые
admin-cases-status = Статус
admin-cases-status-open = Открыто
admin-cases-status-in_review = В работе
admin-cases-status-resolved = Подтверждено
admin-cases-status-rejected = Отклонено
admin-cases-claimed-by = Ведёт
admin-cases-reports = { $reports } жалоб от { $people } человек
admin-cases-empty-title = Дел нет
admin-cases-empty-text = Разбирать нечего — жалобы заводят дело сами.
admin-case-title = Дело
admin-case-server = Сервер
admin-case-reports = Жалобы
admin-case-timeline = Что происходило
admin-case-chat = Чат вокруг события
admin-case-punish = Наказать
admin-case-verdict = Вердикт
admin-case-verdict-confirmed = Подтвердилось
admin-case-verdict-rejected = Не подтвердилось
admin-case-verdict-insufficient = Данных не хватило
admin-case-claim = Взять дело
admin-case-release = Вернуть в очередь
admin-case-probes = Спросить игру
admin-case-probe-chat = Срез чата
admin-case-probe-inventory = Инвентарь
admin-case-probe-client = Проверка клиента
admin-case-note = Заметка
admin-case-note-hint = Что вы увидели, своими словами
admin-case-note-add = Добавить
admin-case-close = Закрыть дело
admin-case-close-do = Закрыть
admin-case-resolution-hint = Что увидит автор жалобы
admin-case-reporter-stats = подтвердилось { $confirmed } из { $total }
admin-case-chat-forbidden = Нет права читать срез чата.
admin-case-chat-empty = В срезе нет сообщений.
admin-case-chat-saved = приложено сообщений: { $count }
admin-case-channel-all = Все
admin-case-channel-public = Общий
admin-case-channel-local = Локальный
admin-case-channel-private = Личные
admin-case-channel-command = Команды
admin-case-event-report_added = Жалоба
admin-case-event-claimed = Взято в работу
admin-case-event-released = Возвращено в очередь
admin-case-event-teleport = Телепорт
admin-case-event-freeze = Заморозка
admin-case-event-watch_start = Начал наблюдение
admin-case-event-watch_stop = Закончил наблюдение
admin-case-event-inventory_snapshot = Снимок инвентаря
admin-case-event-chat_slice = Приложен чат
admin-case-event-client_check = Запрошена проверка клиента
admin-case-event-screenshot = Приложен скриншот
admin-case-event-note = Заметка
admin-case-event-punishment = Выдано наказание
admin-case-event-verdict = Вердикт
admin-case-missing-title = Дело не найдено
admin-case-missing-text = Его удалили или ссылка неверная.
admin-case-source-web = из панели
admin-case-source-game = в игре
admin-case-source-system = автоматически
admin-case-opened = открыто
admin-case-rule = правило
admin-case-report-no-place = место не записано
admin-case-metric-reports = Жалоб
admin-case-metric-people = Человек
admin-case-metric-punishments = Наказаний
admin-case-claim-hint = Запирает дело за вами и открывает меню разбора в игре
admin-case-release-hint = Возвращает дело в очередь — его сможет взять любой
admin-case-probe-chat-hint = Просит сервер прислать чат вокруг жалобы
admin-case-probe-inventory-hint = Снимает то, что у игрока в руках прямо сейчас
admin-case-probe-client-hint = Просит лаунчер игрока проверить сборку
admin-case-verdict-confirmed-hint = Жалоба подтвердилась, наказание остаётся в силе
admin-case-verdict-rejected-hint = Ничего не было, автор жалобы это увидит
admin-case-verdict-insufficient-hint = Доказательств нет ни за, ни против — никого не наказываем
admin-case-trust-title = Подтверждённые, отклонённые и ещё не рассмотренные жалобы игрока
admin-blocklist-presets = Частые читы
admin-blocklist-preset-xray = Известный X-Ray пак
admin-blocklist-preset-client = Чит-клиент
admin-blocklist-preset-bot = Бот автопрохождения
admin-blocklist-preset-combat = Помощь в бою
admin-blocklist-preset-freecam = Свободная камера
admin-blocklist-preset-macro = Автокликер или макрос
admin-blocklist-too-wide = Такая маска совпадёт почти со всем в папке игры
admin-blocklist-bad-hash = SHA1 — это ровно 40 hex-символов
admin-blocklist-sha1-mode-manual = вручную
admin-blocklist-sha1-mode-file = из образца
admin-blocklist-sha1-pick = Выберите файл мода
admin-blocklist-sha1-local = Хэш считается у вас, файл не уходит.
admin-blocklist-sha1-no-crypto = Для подсчёта нужен защищённый контекст (https или localhost)
admin-cases-search = Номер, игрок или сервер
web-date-today = сегодня
web-date-yesterday = вчера
web-date-days = { $count } дн. назад
web-date-months = { $count } мес. назад
web-date-years = { $count } г. назад
nav-admin-backup = Резервная копия
admin-backup-title = Резервная копия
admin-backup-subtitle = Копия мастера, к которой можно вернуться
admin-backup-db = База данных
admin-backup-db-text = Отдаёт pg_dump всей базы: аккаунты, серверы, свод правил, дела и наказания.
admin-backup-download = Скачать дамп
admin-backup-files-note = В голом дампе только база. Сборки, скины, плащи и агенты лежат в томе данных — чтобы забрать и их, берите полный архив.
admin-backup-full = Полный архив
admin-backup-full-text = Дамп базы, все файлы тома данных, паспорт и подпись Ed25519 в одном noro-backup-{ "{" }дата{ "}" }.tar.gz. Собирается на лету — на 4 ГБ данных это примерно полминуты.
admin-backup-full-btn = Собрать полный архив
admin-backup-full-started = Браузер качает архив — следите за списком загрузок.
admin-backup-restore = Восстановление
admin-backup-restore-text = Загрузите архив, чтобы его разобрать. До подтверждения ничего не трогается.
admin-backup-dropzone = Перетащите сюда архив noro-backup или нажмите, чтобы выбрать
admin-backup-dropzone-hint = .tar.gz, собранный этим мастером
admin-backup-inspect-btn = Разобрать архив
admin-backup-uploading = загрузка, { $percent }%
admin-backup-upload-failed = Загрузка не дошла до мастера
admin-backup-created = Собран
admin-backup-master-version = Версия мастера
admin-backup-schema = Версия схемы
admin-backup-schema-pair = архив { $archive }, мастер { $master }
admin-backup-size = Размер
admin-backup-files = Файлов
admin-backup-signature = Подпись
admin-backup-signature-ok = сходится
admin-backup-signature-bad = не сходится с ключом этого мастера
admin-backup-unsigned = архив без подписи
admin-backup-swap-note = Текущие файлы не удаляются, а уезжают в .noro-backup/old-{ "{" }дата{ "}" } внутри тома данных — не тот архив можно откатить руками.
admin-backup-restore-btn = Восстановить из этого архива
admin-backup-confirm-title = Восстановить мастер?
admin-backup-confirm-text = Это заменит базу и все файлы тома данных. После этого мастер перезапустится, чтобы подняться на восстановленных данных, и на несколько секунд станет недоступен.
admin-backup-confirm-btn = Да, восстановить и перезапустить
admin-backup-restore-done = Восстановлено — мастер перезапускается

# Привязки платформ в кабинете
cabinet-identities-title = Привязанные платформы
cabinet-identities-lead = Войти можно любой из них — все ведут в этот аккаунт
cabinet-identities-none-title = Платформы не привязаны
cabinet-identities-none-text = Аккаунт заведён оператором. Привяжите платформу, чтобы входить через неё.
cabinet-identity-primary = Платформа регистрации
cabinet-identity-nothing-to-link = Других способов входа на этом инстансе пока не настроено.
cabinet-identity-unlink = Отвязать
cabinet-identity-link = Привязать { $provider }
cabinet-identity-taken = Этот аккаунт { $provider } уже привязан к другому игроку.

# Админка: способы входа
admin-auth-title = ВХОД
admin-auth-subtitle = Чем игроки могут входить
admin-auth-oauth-lead = Ключи приложения из портала платформы
admin-auth-passkey-lead = Вход без пароля: Touch ID, Face ID или аппаратный ключ
admin-auth-on = Вкл
admin-auth-off = Выкл
admin-auth-not-configured = Включено, но ключи приложения не заданы — кнопка приведёт к отказу.
admin-auth-passkey-blocked = Нужны публичные адреса сайта и API по HTTPS.
admin-auth-secret-set = задан
admin-auth-secret-unset = не задан
admin-auth-secret-placeholder = Пусто — оставить текущий
admin-auth-redirect = Redirect URI для портала платформы
nav-group-auth = ВХОД
nav-group-profile = Профиль
nav-group-account = Аккаунт

login-image-empty = Картинка не задана
admin-settings-login-image-title = Картинка страницы входа
admin-settings-login-image-desc = Показывается рядом с формой входа. GIF и WebP сохраняют анимацию и прозрачность.

login-heading = ВХОД
login-lead = Выберите платформу — вход через любую ведёт в один и тот же аккаунт.
login-recovery-back = Назад к способам входа
login-recovery-lead = Введите ник и один из кодов восстановления.

## OAuth2-приложения
app-status-pending = на проверке
app-status-approved = одобрено
app-status-rejected = отклонено
app-status-suspended = заблокировано
cabinet-apps-connected = Подключённые приложения
cabinet-apps-since = Доступ выдан { $date }
cabinet-apps-revoke-confirm = Отозвать доступ у «{ $name }»? Его токены перестанут работать сразу.
cabinet-myapps-title = Мои приложения
cabinet-myapps-lead = Приложения, которые вы написали сами. Новое проходит проверку, прежде чем в него смогут входить другие игроки.
cabinet-myapps-create = Новое приложение
cabinet-myapps-none-title = Приложений пока нет
cabinet-myapps-none-text = Создайте, чтобы игроки могли входить на ваш сайт или в бота своим аккаунтом отсюда.
cabinet-myapps-name = Название
cabinet-myapps-name-hint = То, что игрок увидит на экране согласия
cabinet-myapps-desc = Описание
cabinet-myapps-desc-hint = Строка о том, что приложение делает
cabinet-myapps-redirects = Адреса возврата
cabinet-myapps-redirects-hint = По одному в строке. Игрок вернётся только на эти адреса.
cabinet-myapps-scopes = Разрешено
cabinet-myapps-pick-scopes = Что приложение будет запрашивать
cabinet-myapps-pick-scopes-hint = Просите минимум: игрок видит этот список, прежде чем пустить вас. Всё сверх него выдаёт персонал.
cabinet-myapps-edit = Изменить
cabinet-myapps-cancel = Отмена
cabinet-myapps-edit-requeues = После сохранения приложение снова уйдёт на проверку.
cabinet-myapps-delete-confirm = Удалить «{ $name }»? Все, кто его подключил, потеряют доступ.
cabinet-myapps-rotate = Выпустить новый секрет
cabinet-myapps-rotate-confirm = Старый секрет перестанет работать сразу. Продолжить?
cabinet-myapps-secret-once = Client secret — показывается один раз
cabinet-myapps-review-note = Модератор
cabinet-myapps-icon-hint = Загрузить иконку: квадрат, от 64×64, до 4 МБ
admin-apps-title = Приложения
admin-apps-subtitle = OAuth2-приложения инстанса
admin-apps-toggles = Сторонние приложения
admin-apps-toggles-lead = На официальные приложения эти переключатели не влияют.
admin-apps-enabled = Сторонние приложения работают
admin-apps-enabled-hint = Выключено: войти через них нельзя, раздел пропадает из кабинетов.
admin-apps-creation = Игроки могут создавать приложения
admin-apps-creation-hint = Выключено: существующие работают, новые создать нельзя.
admin-apps-filter-all = Все
admin-apps-owner = Автор
admin-apps-users = Игроков подключено
admin-apps-scopes = Что может запрашивать
admin-apps-approve = Одобрить
admin-apps-reject = Отклонить
admin-apps-suspend = Заблокировать
admin-apps-delete = Удалить
admin-apps-confirm = Подтвердить
admin-apps-note-placeholder = Причина — её увидит автор
admin-apps-last-note = Последнее решение
admin-apps-delete-confirm = Удалить «{ $name }»? Его токены перестанут работать сразу.
admin-apps-none = Приложений нет
admin-apps-none-hint = Здесь появятся приложения, созданные игроками.
nav-admin-apps = Приложения


# Permission Nodes
perm-group-panel = Панель
perm-group-players = Игроки
perm-group-moderation = Модерация
perm-group-rules = Правила
perm-group-servers = Серверы
perm-group-builds = Сборки
perm-group-machine = Машина
perm-group-other = Действия в игре
perm-group-content = Контент
perm-group-launcher = Лаунчер
perm-group-security = Защита
perm-group-support = Поддержка
perm-group-system = Система
perm-group-player = Игрок
perm-node-admin-stats = Метрики и статистика панели
perm-node-admin-agents = Список статусов агентов
perm-node-users-view = Просмотр списка игроков и профиля
perm-node-users-edit = Редактирование аккаунта и привязок
perm-node-users-username = Смена игрового ника
perm-node-users-delete = Удаление аккаунтов игроков
perm-node-users-roles = Назначение и снятие ролей
perm-node-users-permissions = Прямая выдача персональных прав
perm-node-users-notes-view = Просмотр заметок персонала
perm-node-users-notes-write = Создание заметок персонала
perm-node-users-notes-delete = Удаление заметок персонала
perm-node-users-sessions-view = Просмотр активных сессий
perm-node-users-sessions-kill = Завершение сессий игроков
perm-node-users-journal = Журнал запусков и событий
perm-node-users-launcher = Диагностика и удаленные действия
perm-node-users-skin = Смена скина игроку
perm-node-users-capes = Выдача плащей игроку
perm-node-impersonate = Вход под аккаунтом игрока
perm-node-punish-view = Просмотр истории наказаний
perm-node-punish-warn = Выдача предупреждений (варнов)
perm-node-punish-mute = Выдача блокировок чата (мутов)
perm-node-punish-ban = Выдача глобальных банов
perm-node-punish-server-ban = Бан на конкретном сервере
perm-node-punish-revoke = Снятие наказаний
perm-node-punish-bypass = Обход ограничений правил
perm-node-punish-permanent = Выдача перманентных наказаний
perm-node-freeze = Заморозка игроков
perm-node-reports-view = Просмотр жалоб игроков
perm-node-reports-resolve = Обработка жалоб игроков
perm-node-cases-view = Просмотр очереди и карточек дел
perm-node-cases-claim = Взятие дела в работу
perm-node-cases-resolve = Закрытие дела с вердиктом
perm-node-cases-chat = Просмотр среза чата в деле
perm-node-cases-inventory = Просмотр инвентаря игрока
perm-node-cases-watch = Слежка за игроком
perm-node-cases-client = Проверка лаунчера игрока
perm-node-rules-view = Просмотр свода правил
perm-node-rules-edit = Создание и редактирование правил
perm-node-rules-delete = Удаление правил и разделов
perm-node-servers-view = Просмотр серверов и сборок
perm-node-servers-edit = Создание и редактирование серверов
perm-node-servers-delete = Удаление серверов
perm-node-servers-agents = Игровые серверы и токены агентов
perm-node-servers-roles = Игровые роли сервера
perm-node-builds-view = Просмотр содержимого сборок
perm-node-builds-edit = Редактирование сборок и файлов
perm-node-builds-publish = Публикация сборок игрокам
perm-node-builds-delete = Удаление сборок
perm-node-builds-import = Импорт сборки модов
perm-node-mods-view = Просмотр каталога модов
perm-node-mods-install = Установка модов в сборку
perm-node-mods-remove = Удаление модов из сборки
perm-node-cores-edit = Ядра серверов и загрузчики
perm-node-wrapper-view = Просмотр состояния игровой машины
perm-node-wrapper-console = Чтение консоли сервера
perm-node-wrapper-command = Отправка команд в консоль
perm-node-wrapper-files = Чтение и запись файлов сервера
perm-node-wrapper-power = Управление питанием и перезапуск
perm-node-wrapper-backups = Создание и восстановление бэкапов
perm-node-game-kick = Кик игрока из игры
perm-node-game-tell = Отправка ЛС игроку в игре
perm-node-game-announce = Объявление в игре
perm-node-news-view = Просмотр новостей
perm-node-news-edit = Создание и публикация новостей
perm-node-news-delete = Удаление новостей
perm-node-translations-view = Просмотр переводов интерфейса
perm-node-translations-edit = Редактирование переводов интерфейса
perm-node-capes-view = Просмотр плащей
perm-node-capes-edit = Загрузка и удаление плащей
perm-node-launcher-view = Просмотр релизов лаунчера
perm-node-launcher-publish = Публикация релиза лаунчера
perm-node-launcher-deploy = Выкатка релизов игрокам
perm-node-launcher-clients = Клиенты лаунчера и их статус
perm-node-launcher-tokens = Выдача API-токенов лаунчера
perm-node-integrity-view = Просмотр флагов целостности
perm-node-integrity-review = Разбор флагов целостности
perm-node-blocklist-view = Просмотр черного списка файлов
perm-node-blocklist-edit = Редактирование черного списка
perm-node-chat-filters-view = Просмотр фильтров чата
perm-node-chat-filters-edit = Настройка автомодерации и фильтров
perm-node-chat-filters-delete = Удаление фильтров чата
perm-node-support-logs = Просмотр бандлов логов
perm-node-support-request = Запрос логов у игрока
perm-node-support-force = Принудительный сбор логов
perm-node-support-download = Скачивание бандлов логов
perm-node-support-delete = Удаление логов поддержки
perm-node-moderation-view = Просмотр модераторских шаблонов
perm-node-moderation-edit = Редактирование модераторских шаблонов
perm-node-roles-view = Просмотр ролей
perm-node-roles-edit = Создание и редактирование ролей
perm-node-audit = Просмотр журнала аудита
perm-node-settings-view = Просмотр настроек инстанса
perm-node-settings-edit = Изменение настроек инстанса
perm-node-auth-methods-view = Просмотр способов входа
perm-node-auth-methods-edit = Настройка способов входа
perm-node-tokens-view = Просмотр API-токенов панели
perm-node-tokens-manage = Создание и отзыв API-токенов панели
perm-node-restarts-view = Просмотр расписания перезапусков
perm-node-restarts-edit = Управление расписанием перезапусков
perm-node-oauth-view = Просмотр OAuth2-приложений
perm-node-oauth-manage = Модерация OAuth2-приложений
perm-node-storage = Очистка неиспользуемых объектов стора
perm-node-backup = Скачивание дампа базы данных
perm-node-backup-restore = Восстановление мастера из архива
perm-node-launcher-beta = Бета-канал лаунчера
perm-group-access = Доступ
perm-group-game = Сервер
perm-group-branches = Ветки
perm-superadmin-desc = Суперадмин: Полный неограниченный доступ
perm-admin-all-desc = Админ панели: Все разделы администрирования
