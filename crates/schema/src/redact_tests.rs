//! Корпус из настоящих строк: hs_err, latest.log с Fabric и authlib-injector,
//! лог лаунчера.
//!
//! Главный тест — последний: ни один известный формат токена не проходит
//! наружу. Он и есть смысл модуля, остальные проверяют, что при этом лог
//! остаётся читаемым.

use super::*;

/// Живой токен из корпуса. Ни в одном выводе его быть не должно.
const TOKEN: &str = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NSJ9.Qm9ndXNTaWduYXR1cmU";
const SESSION_UUID: &str = "0f2b6d1a-9c3e-4a77-8b52-1e4d9a6c3f80";

/// Строка из hs_err_pid*.log — там JVM печатает командную строку целиком.
fn hs_err() -> String {
    format!(
        "Command Line: -Xmx4096M -Djava.library.path=C:\\Users\\Дмитрий\\AppData\\noro\\.natives \
         net.minecraft.client.main.Main --username Steve --uuid {SESSION_UUID} \
         --accessToken {TOKEN} --clientId abcdef123456 --userType msa"
    )
}

/// Строка из latest.log: log4j печатает массив аргументов через запятую.
fn latest_log() -> String {
    format!(
        "[12:04:31] [main/INFO]: Program arguments: [--username, Steve, --accessToken, {TOKEN}, \
         --uuid, {SESSION_UUID}, --gameDir, /Users/dmitry/noro/instances/survival]"
    )
}

/// Лог лаунчера: свой формат, свои ключи.
fn launcher_log() -> String {
    format!(
        "2026-08-16T10:02:59Z INFO backend: authenticated discord_id=284712910392819712 \
         session_id={SESSION_UUID} token={TOKEN} master=https://api.example.dev"
    )
}

/// Санитизация возвращает `Cow`, привязанный к входу; тестам удобнее владеть.
fn clean(text: &str) -> String {
    redact(text).into_owned()
}

fn all_samples() -> Vec<String> {
    vec![hs_err(), latest_log(), launcher_log()]
}

#[test]
fn the_command_line_from_hs_err_loses_its_token() {
    let out = clean(&hs_err());
    assert!(!out.contains(TOKEN));
    assert!(out.contains("--accessToken *****"), "{out}");
    // Остальное должно уцелеть: без -Xmx и имени класса файл бесполезен.
    assert!(out.contains("-Xmx4096M"));
    assert!(out.contains("net.minecraft.client.main.Main"));
}

#[test]
fn the_log4j_argument_array_loses_its_token() {
    let out = clean(&latest_log());
    assert!(!out.contains(TOKEN));
    assert!(out.contains("--accessToken, *****"), "{out}");
    assert!(out.contains("--username, Steve"));
}

#[test]
fn player_identifiers_are_masked() {
    for sample in all_samples() {
        let out = clean(&sample);
        assert!(!out.contains(SESSION_UUID), "остался uuid игрока: {out}");
    }
    assert!(!clean(&launcher_log()).contains("284712910392819712"));
}

#[test]
fn home_directories_lose_the_persons_name() {
    let out = clean(&hs_err());
    assert!(!out.contains("Дмитрий"), "{out}");
    assert!(out.contains("\\Users\\*****\\"), "{out}");

    let out = clean(&latest_log());
    assert!(!out.contains("/Users/dmitry/"), "{out}");
    assert!(out.contains("/Users/*****/"), "{out}");
}

#[test]
fn a_bearer_header_is_masked() {
    let out = clean(&format!("GET /api/me Authorization: Bearer {TOKEN}"));
    assert!(!out.contains(TOKEN));
    assert!(out.contains("Bearer *****"), "{out}");
}

#[test]
fn the_last_octet_of_an_address_is_hidden() {
    let out = redact("Connecting to 203.0.113.42:25565");
    assert_eq!(out, "Connecting to 203.0.113.***:25565");
}

#[test]
fn ordinary_lines_survive_untouched() {
    // Санитизация, которая портит обычные строки, приводит к тому, что логи
    // перестают собирать вовсе.
    let line = "[12:04:31] [main/INFO]: Loading 148 mods: fabric-api 0.92.2+1.20.1";
    assert_eq!(clean(line), line);

    let line = "[Render thread/WARN]: Missing sound for event: minecraft:entity.horse.armor";
    assert_eq!(clean(line), line);
}

#[test]
fn no_known_token_format_gets_out() {
    // Тот самый тест, ради которого корпус и собирался.
    let formats = [
        format!("--accessToken {TOKEN}"),
        format!("--accessToken={TOKEN}"),
        format!("\"--accessToken, {TOKEN}\""),
        format!("accessToken: {TOKEN}"),
        format!("access_token={TOKEN}"),
        format!("\"accessToken\":\"{TOKEN}\""),
        format!("Authorization: Bearer {TOKEN}"),
        format!("token={TOKEN}"),
        format!("SignedJWT: {TOKEN}"),
        format!("Session ID is token:{TOKEN}:{SESSION_UUID}"),
        format!("--session {TOKEN}"),
    ];
    for sample in formats {
        let out = clean(&sample);
        assert!(
            !out.contains(TOKEN),
            "токен прошёл наружу: {sample} → {out}"
        );
    }
    for sample in all_samples() {
        let out = clean(&sample);
        assert!(!out.contains(TOKEN), "токен прошёл наружу: {out}");
    }
}
