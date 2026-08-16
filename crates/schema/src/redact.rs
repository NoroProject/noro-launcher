//! Санитизация логов перед тем, как они куда-либо уедут.
//!
//! Утечка тут направлена не от сети к игроку, а наоборот: в логе лежит живой
//! Yggdrasil-токен, и попав в бандл, он окажется в БД и во вьювере у всех, у
//! кого есть доступ. Поэтому правила писались от известных форматов, а не «на
//! глаз», и покрыты корпусом из настоящих файлов.
//!
//! Самое опасное место — `hs_err_pid*.log`: JVM печатает там полную командную
//! строку, то есть `--accessToken` целиком.

use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;

/// Чем заменяется вырезанное. Одинаковая длина у всех замен нарочно: по длине
/// маски не должно быть видно, что именно за ней стояло.
const MASK: &str = "*****";

/// Правила применяются по порядку: сначала точные форматы, потом общие. Общее
/// правило не срабатывает повторно — маска не похожа на секрет.
static RULES: Lazy<Vec<(Regex, String)>> = Lazy::new(|| {
    let rules: Vec<(&str, String)> = vec![
        // Управляющие последовательности терминала из консольного вывода log4j.
        (r"\x1b\[[0-9;?]*[ -/]*[@-~]", String::new()),
        // Аргументы запуска. Три формы сразу: `--accessToken abc` (hs_err
        // печатает строку целиком), `--accessToken=abc` и `"--accessToken, abc"`
        // (log4j печатает массив аргументов через запятую).
        (
            r#"(?i)(--(?:accessToken|session|uuid|clientId|xuid))([=,]?\s*"?)([^\s,"\]]+)"#,
            format!("$1$2{MASK}"),
        ),
        // Свободный текст: `token: abc`, `Bearer abc`, `"session_id":"abc"`.
        (
            r#"(?i)\b(bearer|access[_-]?token|session[_-]?id|api[_-]?key|token|session)(["']?\s*[:=]\s*["']?|\s+)([A-Za-z0-9._\-]{8,})"#,
            format!("$1$2{MASK}"),
        ),
        // Форматы, которые печатает сам клиент.
        (r"(?i)SignedJWT:\s*\S+", format!("SignedJWT: {MASK}")),
        (r"(?i)Session ID is \S+", format!("Session ID is {MASK}")),
        // Идентификаторы игрока: они не секрет, но связывают лог с человеком.
        (
            r#"(?i)\b(uuid|profileId|playerUUID|user)(["']?\s*[:=]\s*["']?\{?)([0-9a-f]{8}-?[0-9a-f]{4}-?[0-9a-f]{4}-?[0-9a-f]{4}-?[0-9a-f]{12})"#,
            format!("$1$2{MASK}"),
        ),
        (
            r#"(?i)(discord[_a-z]*["']?\s*[:=]\s*["']?)(\d{17,20})"#,
            format!("$1{MASK}"),
        ),
        // Домашний каталог: в нём почти всегда настоящее имя человека.
        (r"/home/[^/]+/", format!("/home/{MASK}/")),
        (r"/Users/[^/]+/", format!("/Users/{MASK}/")),
        // Двойные слэши идут первыми: путь из JSON приходит экранированным, и
        // правило для одинарных съело бы его половину.
        (
            r"\\\\Users\\\\[^\\]+\\\\",
            format!("\\\\Users\\\\{MASK}\\\\"),
        ),
        (r"\\Users\\[^\\]+\\", format!("\\Users\\{MASK}\\")),
        // IPv4: последний октет скрывается, сеть остаётся — по ней ещё можно
        // понять, что игрок и сервер в одной подсети.
        (r"\b(\d{1,3}\.\d{1,3}\.\d{1,3})\.\d{1,3}\b", "$1.***".into()),
    ];

    rules
        .into_iter()
        .map(|(pattern, replacement)| {
            // Регулярки заданы в коде: невалидная — это опечатка разработчика,
            // и узнать о ней надо сразу, а не по утёкшему токену.
            (
                Regex::new(pattern).expect("правило санитизации"),
                replacement,
            )
        })
        .collect()
});

/// Очистить строку от чувствительных данных.
pub fn redact(text: &str) -> Cow<'_, str> {
    let mut out = Cow::Borrowed(text);
    for (regex, replacement) in RULES.iter() {
        if let Cow::Owned(replaced) = regex.replace_all(&out, replacement.as_str()) {
            out = Cow::Owned(replaced);
        }
    }
    out
}

#[cfg(test)]
#[path = "redact_tests.rs"]
mod tests;
