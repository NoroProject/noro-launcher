//! Ключи настроек, живущих в БД.
//!
//! У каждой — переменная окружения, которая её перекрывает. Приоритет
//! `env > БД > дефолт` нужен ради боевого инстанса: он поднят через compose, и
//! перестать слушаться собственного окружения он не должен.

/// Настройка, которая может прийти из env или из БД.
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
    HERO_IMAGE_URL  => "hero_image_url",  "NORO_HERO_IMAGE_URL";
    DISCORD_CLIENT_ID => "discord_client_id", "DISCORD_CLIENT_ID";
    GITHUB_REPO     => "github_repo",     "NORO_GITHUB_REPO";
    GITHUB_REF      => "github_ref",      "NORO_GITHUB_REF";
    LAUNCHER_REPO   => "launcher_repo",   "NORO_LAUNCHER_REPO";
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
