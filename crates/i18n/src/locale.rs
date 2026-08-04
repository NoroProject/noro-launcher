//! Поддерживаемые языки.

use unic_langid::LanguageIdentifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Locale {
    #[default]
    En,
    Ru,
}

/// Встроенные каталоги — фолбэк, когда мастер недоступен или не знает язык.
const EN_FTL: &str = include_str!("../locales/en.ftl");
const RU_FTL: &str = include_str!("../locales/ru.ftl");

impl Locale {
    pub const ALL: [Locale; 2] = [Locale::En, Locale::Ru];

    /// Код языка для API мастера и для сохранения в конфиге.
    pub fn code(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ru => "ru",
        }
    }

    /// Подпись для переключателя в рамке окна.
    pub fn label(self) -> &'static str {
        match self {
            Locale::En => "ENG",
            Locale::Ru => "RU",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        // Принимаем и региональные варианты: `ru-RU` — это `ru`.
        let base = code.split(['-', '_']).next().unwrap_or(code);
        match base.to_ascii_lowercase().as_str() {
            "en" => Some(Locale::En),
            "ru" => Some(Locale::Ru),
            _ => None,
        }
    }

    pub(crate) fn lang_id(self) -> LanguageIdentifier {
        // Коды заданы константами выше, так что разбор не может не пройти.
        self.code().parse().expect("валидный код языка")
    }

    /// Сырой текст встроенного каталога — админке нужен как эталон,
    /// чтобы показать, какие ключи вообще существуют.
    pub fn builtin_ftl(self) -> &'static str {
        match self {
            Locale::En => EN_FTL,
            Locale::Ru => RU_FTL,
        }
    }
}
