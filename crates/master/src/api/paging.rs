//! Общий вид списочных ручек: поиск, страница, счётчик.
//!
//! До этого каждый список отдавал голый массив и брал что придётся: где-то
//! `limit/offset`, где-то ничего. Клиент тянул всё и фильтровал у себя — и
//! молча терял то, что не поместилось в первую выдачу. Так спотлайт не находил
//! игрока за пределами первой сотни.
//!
//! Отсюда единая форма: `{ items, total }`. `total` считается по тому же
//! условию, что и выборка, — без него в интерфейсе нельзя показать ни номера
//! страниц, ни «найдено 348», а «показать ещё» не знает, когда остановиться.

use serde::Deserialize;

/// Сам конверт `{items, total}` живёт в `schema`: его читают ещё лаунчер и CLI,
/// и объявлять его форму отдельно на каждой стороне значит дать им разойтись.
pub use schema::Page;

/// Сколько отдаём, если клиент не попросил иначе.
const DEFAULT_LIMIT: i64 = 50;
/// Потолок. Запрос `?limit=100000` — это не пагинация, а выгрузка всей таблицы
/// в один ответ, и упереться в него должен сервер, а не память процесса.
const MAX_LIMIT: i64 = 200;

/// Общие параметры списочной ручки.
///
/// Числа принимаются и числами, и строками — см. `flexible_i64`.
#[derive(Debug, Default, Deserialize)]
pub struct PageQuery {
    /// Строка поиска. Что именно ищется — дело конкретного запроса.
    pub q: Option<String>,
    #[serde(default, deserialize_with = "flexible_i64::deserialize")]
    pub limit: Option<i64>,
    #[serde(default, deserialize_with = "flexible_i64::deserialize")]
    pub offset: Option<i64>,
}

impl PageQuery {
    /// Собрать из полей ручки, у которой есть и свои фильтры.
    pub fn from_parts(q: Option<String>, limit: Option<i64>, offset: Option<i64>) -> Self {
        Self { q, limit, offset }
    }

    /// Сколько строк отдавать. Всегда в пределах `1..=MAX_LIMIT`.
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }

    /// Отрицательный сдвиг Postgres не примет, поэтому режется здесь.
    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    /// Непустая строка поиска. Пробелы и пустая строка — это «без поиска», а не
    /// «искать пустоту»: иначе очистка поля выдавала бы пустой список.
    pub fn search(&self) -> Option<&str> {
        self.q.as_deref().map(str::trim).filter(|s| !s.is_empty())
    }

    /// Шаблон для `ILIKE ... ESCAPE '\'`.
    ///
    /// `%` и `_` из запроса экранируются: ник `_` иначе означал бы «любой
    /// символ» и выдавал всех подряд, а `%` — вообще всех.
    pub fn like(&self) -> Option<String> {
        self.search().map(|s| format!("%{}%", escape_like(s)))
    }
}

/// Число, пришедшее числом, строкой или не пришедшее вовсе.
///
/// Query-строка текстовая по природе: `?limit=25` — это всегда символы, и то,
/// что serde обычно разбирает их в `i64` сам, — удобная случайность, а не
/// гарантия. Стоит попасть на путь, где значение буферизуется (`flatten`,
/// `untagged`, чужой клиент, двойное кодирование) — и строгий `Option<i64>`
/// отвечает «invalid type: string "25", expected i64», то есть отказывается
/// понимать собственный же параметр.
///
/// Пустое значение (`?limit=`) — это «не задано», а не ошибка: так его шлёт
/// форма с очищенным полем.
pub mod flexible_i64 {
    use serde::{de, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(d: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Num(i64),
            Text(String),
        }

        match Option::<Raw>::deserialize(d)? {
            None => Ok(None),
            Some(Raw::Num(n)) => Ok(Some(n)),
            Some(Raw::Text(s)) if s.trim().is_empty() => Ok(None),
            Some(Raw::Text(s)) => s.trim().parse().map(Some).map_err(de::Error::custom),
        }
    }
}

/// Обезвредить подстановочные знаки внутри пользовательской строки.
pub fn escape_like(needle: &str) -> String {
    needle
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
#[path = "paging_tests.rs"]
mod tests;
