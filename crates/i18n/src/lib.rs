//! Локализация лаунчера на Fluent.
//!
//! Формат `.ftl` выбран из-за встроенных селекторов множественного числа: в
//! русском их три (one/few/many), и в JSON/YAML/TOML это пришлось бы городить
//! руками. Каталоги вшиты в бинарник и могут переопределяться с мастера.
//!
//! Порядок поиска ключа: каталог с мастера → встроенный каталог языка →
//! английский → сам ключ. Так недопереведённый интерфейс остаётся рабочим.

mod catalog;
mod locale;

pub use catalog::Catalog;
pub use fluent_bundle::FluentArgs;
pub use locale::Locale;

use parking_lot::RwLock;

/// Активный каталог процесса. Глобальный, потому что перевод нужен из любого
/// места отрисовки, а прокидывать его через каждый компонент — это то самое
/// prop-drilling, которого мы избегаем.
static ACTIVE: RwLock<Option<Active>> = RwLock::new(None);

struct Active {
    /// Каталог с мастера. Он почти всегда частичный — правят одну-две строки,
    /// поэтому он именно накладывается поверх встроенного, а не заменяет его.
    patch: Option<Catalog>,
    /// Встроенный каталог выбранного языка.
    builtin: Catalog,
    /// Английский всегда под рукой как последний рубеж перед показом ключа.
    fallback: Catalog,
}

/// Переключить язык на встроенный каталог.
pub fn set_locale(locale: Locale) {
    *ACTIVE.write() = Some(Active {
        patch: None,
        builtin: Catalog::builtin(locale),
        fallback: Catalog::builtin(Locale::En),
    });
}

/// Заменить каталог языка присланным с мастера. Возвращает `false`, если
/// текст не разобрался — тогда остаётся действующий каталог.
pub fn install_catalog(locale: Locale, ftl: &str) -> bool {
    let Some(catalog) = Catalog::from_ftl(locale, ftl) else {
        return false;
    };
    let mut guard = ACTIVE.write();
    let (builtin, fallback) = match guard.take() {
        Some(active) => (active.builtin, active.fallback),
        None => (Catalog::builtin(locale), Catalog::builtin(Locale::En)),
    };
    *guard = Some(Active {
        patch: Some(catalog),
        builtin,
        fallback,
    });
    true
}

pub fn locale() -> Locale {
    ACTIVE
        .read()
        .as_ref()
        .map_or(Locale::En, |a| a.builtin.locale())
}

/// Перевести ключ. Неизвестный ключ возвращается как есть — это заметно в UI,
/// но не роняет экран и сразу показывает, чего не хватает в каталоге.
pub fn t(key: &str) -> String {
    lookup(key, None).unwrap_or_else(|| key.to_string())
}

/// Проверить, существует ли ключ перевода в активном или встроенном каталоге.
pub fn has_key(key: &str) -> bool {
    let guard = ACTIVE.read();
    let Some(active) = guard.as_ref() else {
        return false;
    };
    active.patch.as_ref().is_some_and(|p| p.has(key))
        || active.builtin.has(key)
        || active.fallback.has(key)
}

/// Перевести ключ с аргументами (подстановки и множественное число).
pub fn t_args(key: &str, args: &FluentArgs) -> String {
    lookup(key, Some(args)).unwrap_or_else(|| key.to_string())
}

/// Удобная обёртка для самого частого случая — счётчика.
pub fn t_count(key: &str, count: i64) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    t_args(key, &args)
}

fn lookup(key: &str, args: Option<&FluentArgs>) -> Option<String> {
    let guard = ACTIVE.read();
    let active = guard.as_ref()?;
    active
        .patch
        .as_ref()
        .and_then(|p| p.get(key, args))
        .or_else(|| active.builtin.get(key, args))
        .or_else(|| active.fallback.get(key, args))
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
