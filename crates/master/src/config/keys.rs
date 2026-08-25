//! Ключи настроек, живущих в БД.
//!
//! У каждой — переменная окружения, которая её перекрывает. Приоритет
//! `env > БД > дефолт` нужен ради боевого инстанса: он поднят через compose, и
//! перестать слушаться собственного окружения он не должен.

/// Настройка, которая может прийти из env или из БД.
#[derive(Clone, Copy)]
pub struct Key {
    /// Ключ в `instance_settings`.
    pub name: &'static str,
    /// Переменная окружения, перекрывающая значение из БД.
    pub env: &'static str,
}

macro_rules! keys {
    ($($ident:ident => $name:literal, $env:literal;)*) => {
        $(pub const $ident: Key = Key { name: $name, env: $env };)*
        /// Все ключи — для миграции env→БД и для страницы настроек.
        pub const ALL: &[Key] = &[$($ident),*];
    };
}

keys! {
    PUBLIC_URL      => "public_url",      "NORO_PUBLIC_URL";
    WEB_URL         => "web_url",         "NORO_WEB_URL";
    INSTANCE_NAME   => "instance_name",   "NORO_INSTANCE_NAME";
    ALLOWED_ORIGINS => "allowed_origins", "NORO_ALLOWED_ORIGINS";
    FILES_CDN_URL   => "files_cdn_url",   "NORO_FILES_CDN_URL";
    LOGO_URL        => "logo_url",        "NORO_LOGO_URL";
    HERO_IMAGE_URL  => "hero_image_url",  "NORO_HERO_IMAGE_URL";
    LOGIN_IMAGE_URL => "login_image_url", "NORO_LOGIN_IMAGE_URL";
    GITHUB_REPO     => "github_repo",     "NORO_GITHUB_REPO";
    GITHUB_REF      => "github_ref",      "NORO_GITHUB_REF";
    LAUNCHER_REPO   => "launcher_repo",   "NORO_LAUNCHER_REPO";
}

/// Тумблеры сторонних OAuth2-приложений.
///
/// Не в `ALL`: это переключатели, а не строки конфигурации — их место на своей
/// странице в админке, а текстовым полем «true/false» среди адресов и токенов
/// они выглядели бы опечаткой, ожидающей своего часа.
pub const OAUTH_APPS_ENABLED: &str = "oauth_apps_enabled";
pub const OAUTH_APPS_CREATION: &str = "oauth_apps_creation_enabled";

/// Служебная запись «в этой картинке есть прозрачные места»: сайт по ней решает,
/// рисовать ли под иллюстрацией подложку с рамкой. Не настройка и потому не в
/// `ALL`: значение считает загрузчик по самим пикселям, руками его задавать
/// нечего, а лишнее поле на странице настроек только путало бы.
pub fn transparency_key(image_key: &str) -> String {
    format!("{image_key}_transparent")
}

/// Секреты. В БД они не переезжают никогда: секрет в БД — это секрет в дампе,
/// в бэкапе и в реплике. Здесь только имена — чтобы админка могла честно
/// сказать «задано в окружении» либо «не задано», не показывая значения.
pub const SECRET_ENV: &[&str] = &[
    "DISCORD_CLIENT_SECRET",
    "NORO_SIGNING_KEY",
    "GITHUB_TOKEN",
    "CURSEFORGE_API_KEY",
    "NORO_S3_ACCESS_KEY",
    "NORO_S3_SECRET_KEY",
    "SENTRY_DSN",
];
