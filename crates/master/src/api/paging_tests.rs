//! Клампы и экранирование — то, через что в списки прилетает пользовательский
//! ввод. Ошибка здесь либо выгружает таблицу целиком, либо ломает поиск.

use super::*;

fn q(raw: &str) -> PageQuery {
    serde_json::from_str(raw).unwrap()
}

#[test]
fn пустой_запрос_берёт_умолчания() {
    let p = q("{}");
    assert_eq!(p.limit(), DEFAULT_LIMIT);
    assert_eq!(p.offset(), 0);
    assert_eq!(p.search(), None);
    assert_eq!(p.like(), None);
}

#[test]
fn лимит_зажат_сверху_и_снизу() {
    // Выгрузка всей таблицы одним ответом — не пагинация.
    assert_eq!(q(r#"{"limit":100000}"#).limit(), MAX_LIMIT);
    // Ноль и минус Postgres не примет, а пустая страница ещё и бессмысленна.
    assert_eq!(q(r#"{"limit":0}"#).limit(), 1);
    assert_eq!(q(r#"{"limit":-5}"#).limit(), 1);
    assert_eq!(q(r#"{"limit":50}"#).limit(), 50);
}

#[test]
fn отрицательный_сдвиг_не_доходит_до_базы() {
    assert_eq!(q(r#"{"offset":-10}"#).offset(), 0);
    assert_eq!(q(r#"{"offset":120}"#).offset(), 120);
}

#[test]
fn пробелы_это_отсутствие_поиска() {
    // Иначе очистка поля выдавала бы пустой список вместо полного.
    assert_eq!(q(r#"{"q":"   "}"#).search(), None);
    assert_eq!(q(r#"{"q":""}"#).search(), None);
    assert_eq!(q(r#"{"q":"  ник  "}"#).search(), Some("ник"));
}

#[test]
fn подстановочные_знаки_обезвреживаются() {
    // Без экранирования «_» матчит любой символ, а «%» — вообще всё: поиск
    // молча превращался бы в выдачу всей таблицы.
    assert_eq!(q(r#"{"q":"_"}"#).like().unwrap(), r"%\_%");
    assert_eq!(q(r#"{"q":"%"}"#).like().unwrap(), r"%\%%");
    assert_eq!(q(r#"{"q":"a_b%c"}"#).like().unwrap(), r"%a\_b\%c%");
    // Обратный слэш экранируется первым, иначе он съел бы следующий escape.
    assert_eq!(escape_like(r"\_"), r"\\\_");
    assert_eq!(q(r#"{"q":"Dalyn"}"#).like().unwrap(), "%Dalyn%");
}

#[test]
fn настоящая_query_строка_разбирается() {
    // Тем же разборщиком, что и у axum: на JSON поломка не воспроизводилась.
    let p: PageQuery = serde_urlencoded::from_str("limit=25&offset=50").unwrap();
    assert_eq!(p.limit(), 25);
    assert_eq!(p.offset(), 50);

    let p: PageQuery = serde_urlencoded::from_str("q=Dalyn&limit=10").unwrap();
    assert_eq!(p.search(), Some("Dalyn"));
    assert_eq!(p.limit(), 10);

    // Без параметров — умолчания, а не отказ.
    let p: PageQuery = serde_urlencoded::from_str("").unwrap();
    assert_eq!(p.limit(), DEFAULT_LIMIT);
}

#[test]
fn число_принимается_в_любом_виде() {
    // Строкой — так его отдаёт query-строка на путях с буферизацией.
    assert_eq!(q(r#"{"limit":"25"}"#).limit(), 25);
    assert_eq!(q(r#"{"offset":"50"}"#).offset(), 50);
    // Числом — обычный путь.
    assert_eq!(q(r#"{"limit":25}"#).limit(), 25);
    // Пустое значение (`?limit=`) — «не задано», а не отказ: так его шлёт
    // форма с очищенным полем.
    assert_eq!(q(r#"{"limit":""}"#).limit(), DEFAULT_LIMIT);
    assert_eq!(q(r#"{"limit":null}"#).limit(), DEFAULT_LIMIT);
    // Клампы работают и для строкового варианта.
    assert_eq!(q(r#"{"limit":"100000"}"#).limit(), MAX_LIMIT);
    assert_eq!(q(r#"{"offset":"-5"}"#).offset(), 0);
}

/// Флаттен из кода убран, но проверяем и его: вернётся — разбор не сломается.
#[derive(Debug, serde::Deserialize)]
struct СквозьФлаттен {
    #[serde(flatten)]
    page: PageQuery,
}

#[test]
fn флаттен_больше_не_ломает_разбор() {
    let f: СквозьФлаттен = serde_urlencoded::from_str("limit=25&offset=50").unwrap();
    assert_eq!(f.page.limit(), 25);
    assert_eq!(f.page.offset(), 50);

    let f: СквозьФлаттен = serde_json::from_str(r#"{"limit":"25"}"#).unwrap();
    assert_eq!(f.page.limit(), 25);
}

#[test]
fn мусор_вместо_числа_это_отказ_а_не_молчание() {
    // Тихо подставить умолчание нельзя: клиент считает, что попросил страницу,
    // и разойтись с сервером в этом хуже, чем получить внятный отказ.
    assert!(serde_urlencoded::from_str::<PageQuery>("limit=abc").is_err());
    assert!(serde_urlencoded::from_str::<PageQuery>("offset=2.5").is_err());
}

#[test]
fn запрос_страницы_игроков_разбирается_целиком() {
    // Та самая ручка, которая отказывалась открываться. Проверяем её структуру,
    // а не копию: разойдутся — тест это и покажет.
    use crate::api::admin::users::UsersQuery;

    let q: UsersQuery = serde_urlencoded::from_str("limit=25&offset=0").unwrap();
    assert_eq!(q.limit, Some(25));
    assert_eq!(q.banned, None);

    // Свои фильтры тоже приезжают строками — им нужно то же лечение.
    let q: UsersQuery =
        serde_urlencoded::from_str("limit=25&offset=25&q=dal&banned=true&role=admin").unwrap();
    assert_eq!(q.offset, Some(25));
    assert_eq!(q.q.as_deref(), Some("dal"));
    assert_eq!(q.banned, Some(true));
    assert_eq!(q.role.as_deref(), Some("admin"));

    let q: UsersQuery = serde_urlencoded::from_str("banned=false").unwrap();
    assert_eq!(q.banned, Some(false));
}

#[test]
fn страница_несёт_общий_счётчик_а_не_длину_выдачи() {
    let page = Page::new(vec![1, 2, 3], 348);
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.total, 348);

    // Короткий список отдаётся целиком, но форма ответа та же.
    let whole = Page::whole(vec!["a", "b"]);
    assert_eq!(whole.total, 2);
}

#[test]
fn map_сохраняет_счётчик() {
    let page = Page::new(vec![1, 2], 99).map(|n| n * 10);
    assert_eq!(page.items, vec![10, 20]);
    assert_eq!(page.total, 99, "перекладка типов не должна терять total");
}
