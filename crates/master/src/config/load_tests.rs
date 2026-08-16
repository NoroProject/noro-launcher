//! Приоритет источников — то место, где ошибка тише всего: инстанс просто
//! начинает слушаться не того, чего ждёт оператор.

use super::*;
use serde_json::json;

/// Переменные окружения глобальны для процесса, а тесты идут параллельно.
/// Имя переменной своё на каждый тест — иначе они видят чужие значения.
fn key(env: &'static str) -> Key {
    Key {
        name: "public_url",
        env,
    }
}

fn db(value: Value) -> BTreeMap<String, Value> {
    BTreeMap::from([("public_url".to_string(), value)])
}

#[test]
fn env_wins_over_the_database() {
    // Боевой инстанс поднят через compose и обязан слушаться своего окружения.
    let k = key("NORO_TEST_PRIORITY_1");
    std::env::set_var(k.env, "https://from-env.dev");

    let got = setting(&db(json!("https://from-db.dev")), k);

    assert_eq!(got.as_deref(), Some("https://from-env.dev"));
    std::env::remove_var("NORO_TEST_PRIORITY_1");
}

#[test]
fn the_database_is_used_when_env_is_absent() {
    let got = setting(&db(json!("https://from-db.dev")), key("NORO_TEST_ABSENT_2"));
    assert_eq!(got.as_deref(), Some("https://from-db.dev"));
}

#[test]
fn nothing_anywhere_means_none() {
    let got = setting(&BTreeMap::new(), key("NORO_TEST_ABSENT_3"));
    assert_eq!(got, None);
}

#[test]
fn an_empty_env_value_does_not_shadow_the_database() {
    // В docker-compose незаполненный ${VAR} разворачивается в пустую строку, и
    // раньше она молча означала бы «настройка снята».
    let k = key("NORO_TEST_EMPTY_4");
    std::env::set_var(k.env, "   ");

    let got = setting(&db(json!("https://from-db.dev")), k);

    assert_eq!(got.as_deref(), Some("https://from-db.dev"));
    std::env::remove_var("NORO_TEST_EMPTY_4");
}

#[test]
fn an_empty_row_in_the_database_reads_as_unset() {
    let got = setting(&db(json!("")), key("NORO_TEST_ABSENT_5"));
    assert_eq!(got, None);
}

#[test]
fn a_non_string_value_is_still_read() {
    // Руками в JSONB легко положить число; молчать об этом хуже, чем прочитать.
    let got = setting(&db(json!(8080)), key("NORO_TEST_ABSENT_6"));
    assert_eq!(got.as_deref(), Some("8080"));
}

#[test]
fn a_json_null_reads_as_unset() {
    let got = setting(&db(json!(null)), key("NORO_TEST_ABSENT_7"));
    assert_eq!(got, None);
}
