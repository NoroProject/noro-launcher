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
       *[other] { $count } серверов
    }

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
profile-skin-invalid = Подходящий скин не выбран

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

## Уведомления от мастера
notif-server-error = Ошибка сервера: { $reason }
notif-no-server-access = Нет доступа к этому серверу
notif-no-published-build = У сервера нет опубликованной сборки
notif-update-failed = Не удалось обновиться: { $reason }
notif-skin-upload-failed = Не удалось загрузить скин: { $reason }
notif-sign-in-to-upload = Для загрузки скина нужно войти
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

## Панель игры
game-build = СБОРКА
game-start = ЗАПУСТИТЬ
game-install = УСТАНОВИТЬ
game-update = ОБНОВИТЬ
game-stop = ОСТАНОВИТЬ
game-preparing = ПОДГОТОВКА
game-locked = НЕТ ДОСТУПА
game-vip-only = ТОЛЬКО VIP
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
settings-reset = СБРОС
settings-update = ОБНОВЛЕНИЕ ЛАУНЧЕРА
settings-install-update = Установить обновление
settings-source-default = ПО УМОЛЧАНИЮ
settings-source-override = ЛОКАЛЬНАЯ НАСТРОЙКА
settings-source-recommended = РЕКОМЕНДАЦИЯ МАСТЕРА
settings-server = Сервер
console-title = КОНСОЛЬ ИГРЫ
settings-update-hint = Доступна новая версия лаунчера.
