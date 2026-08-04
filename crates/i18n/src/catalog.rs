//! Каталог сообщений одного языка поверх FluentBundle.

use crate::locale::Locale;
use fluent_bundle::{concurrent::FluentBundle, FluentArgs, FluentResource};
use std::borrow::Cow;

/// Разобранный каталог. Строится из встроенного `.ftl` и, если мастер прислал
/// свою версию, из неё — с откатом на встроенную при ошибке разбора.
pub struct Catalog {
    locale: Locale,
    bundle: FluentBundle<FluentResource>,
}

impl Catalog {
    /// Каталог из вшитого в бинарник текста. Не может не собраться: он
    /// проверяется тестами при сборке.
    pub fn builtin(locale: Locale) -> Self {
        Self::parse(locale, locale.builtin_ftl()).unwrap_or_else(|| Self {
            locale,
            bundle: new_bundle(locale),
        })
    }

    /// Каталог из текста, полученного с мастера. `None` — если `.ftl` не
    /// разобрался; вызывающий должен остаться на предыдущем каталоге.
    pub fn from_ftl(locale: Locale, ftl: &str) -> Option<Self> {
        Self::parse(locale, ftl)
    }

    fn parse(locale: Locale, ftl: &str) -> Option<Self> {
        let resource = match FluentResource::try_new(ftl.to_string()) {
            Ok(r) => r,
            Err((r, errors)) => {
                // Fluent возвращает то, что успел разобрать. Сообщения до
                // ошибки рабочие, поэтому каталог не выбрасываем.
                tracing::warn!(locale = locale.code(), ?errors, "ошибки разбора .ftl");
                r
            }
        };
        let mut bundle = new_bundle(locale);
        bundle.add_resource(resource).ok()?;
        Some(Self { locale, bundle })
    }

    pub fn locale(&self) -> Locale {
        self.locale
    }

    pub fn has(&self, key: &str) -> bool {
        self.bundle.has_message(key)
    }

    /// Текст сообщения. `None`, если ключа нет — так вызывающий может
    /// откатиться на английский каталог.
    pub fn get(&self, key: &str, args: Option<&FluentArgs>) -> Option<String> {
        let msg = self.bundle.get_message(key)?;
        let pattern = msg.value()?;
        let mut errors = Vec::new();
        let text = self.bundle.format_pattern(pattern, args, &mut errors);
        if !errors.is_empty() {
            tracing::warn!(
                key,
                locale = self.locale.code(),
                ?errors,
                "ошибка подстановки"
            );
        }
        Some(match text {
            Cow::Borrowed(s) => s.to_string(),
            Cow::Owned(s) => s,
        })
    }
}

fn new_bundle(locale: Locale) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new_concurrent(vec![locale.lang_id()]);
    // Fluent по умолчанию оборачивает подстановки в U+2068/U+2069 (изоляты
    // направления). В GPUI они рисуются как «тофу», а RTL-языков у нас нет.
    bundle.set_use_isolating(false);
    bundle
}
