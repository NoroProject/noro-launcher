//! Тесты бьют по `Catalog` напрямую, а не по глобальному состоянию: тесты
//! идут параллельно и затирали бы друг другу активный язык.

use super::*;

fn count_of(catalog: &Catalog, key: &str, n: i64) -> String {
    let mut args = FluentArgs::new();
    args.set("count", n);
    catalog.get(key, Some(&args)).expect("ключ есть в каталоге")
}

#[test]
fn russian_picks_all_three_plural_forms() {
    let ru = Catalog::builtin(Locale::Ru);
    // Ровно то, ради чего взят Fluent: в JSON/YAML этих правил не выразить.
    assert_eq!(count_of(&ru, "sidebar-servers", 1), "1 сервер"); // one
    assert_eq!(count_of(&ru, "sidebar-servers", 2), "2 сервера"); // few
    assert_eq!(count_of(&ru, "sidebar-servers", 5), "5 серверов"); // many
                                                                   // Подвох русского: 11–14 идут в many, хотя оканчиваются на 1–4.
    assert_eq!(count_of(&ru, "sidebar-servers", 11), "11 серверов");
    assert_eq!(count_of(&ru, "sidebar-servers", 13), "13 серверов");
    // А 21 и 22 возвращаются к one и few.
    assert_eq!(count_of(&ru, "sidebar-servers", 21), "21 сервер");
    assert_eq!(count_of(&ru, "sidebar-servers", 22), "22 сервера");
    assert_eq!(count_of(&ru, "sidebar-servers", 0), "0 серверов");
}

#[test]
fn english_has_only_two_forms() {
    let en = Catalog::builtin(Locale::En);
    assert_eq!(count_of(&en, "sidebar-servers", 1), "1 server");
    assert_eq!(count_of(&en, "sidebar-servers", 2), "2 servers");
    assert_eq!(count_of(&en, "sidebar-servers", 11), "11 servers");
}

#[test]
fn both_catalogs_define_the_same_keys() {
    // Разъехавшиеся каталоги — самая частая поломка перевода: ключ добавили в
    // один язык и забыли в другой, а всплывает это уже у пользователя.
    let en = Catalog::builtin(Locale::En);
    let ru = Catalog::builtin(Locale::Ru);
    let missing: Vec<&str> = keys_of(include_str!("../locales/en.ftl"))
        .into_iter()
        .filter(|k| !ru.has(k))
        .collect();
    assert!(missing.is_empty(), "Missing in ru.ftl: {missing:?}");

    let extra: Vec<&str> = keys_of(include_str!("../locales/ru.ftl"))
        .into_iter()
        .filter(|k| !en.has(k))
        .collect();
    assert!(extra.is_empty(), "Missing in en.ftl: {extra:?}");
}

#[test]
fn a_master_catalog_overrides_the_builtin_one() {
    let patched = Catalog::from_ftl(Locale::Ru, "profile-sign-out = Отключиться")
        .expect("валидный ftl разбирается");
    assert_eq!(
        patched.get("profile-sign-out", None).as_deref(),
        Some("Отключиться")
    );
}

#[test]
fn a_broken_catalog_never_takes_down_the_ui() {
    // Мастер может прислать битый .ftl — интерфейс обязан пережить это.
    let broken = Catalog::from_ftl(Locale::Ru, "= = =\n{{{");
    if let Some(c) = broken {
        assert!(c.get("profile-title", None).is_none());
    }
}

#[test]
fn locale_codes_accept_regional_variants() {
    assert_eq!(Locale::from_code("ru-RU"), Some(Locale::Ru));
    assert_eq!(Locale::from_code("en_US"), Some(Locale::En));
    assert_eq!(Locale::from_code("EN"), Some(Locale::En));
    assert_eq!(Locale::from_code("de"), None);
}

/// Имена сообщений верхнего уровня из текста `.ftl`.
fn keys_of(ftl: &'static str) -> Vec<&'static str> {
    ftl.lines()
        .filter(|l| !l.starts_with([' ', '#', '\t']))
        .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim()))
        .filter(|k| !k.is_empty())
        .collect()
}

#[test]
fn the_global_api_actually_switches_language() {
    // Тесты выше проверяют Catalog напрямую, а UI ходит через глобальные
    // set_locale/t — этот путь до сих пор не был покрыт.
    set_locale(Locale::Ru);
    assert_eq!(locale(), Locale::Ru);
    assert_eq!(t("game-no-servers"), "Серверов нет");

    // Каталог с мастера должен перебивать встроенный.
    assert!(install_catalog(Locale::Ru, "game-no-servers = С мастера"));
    assert_eq!(t("game-no-servers"), "С мастера");

    // Частичный каталог мастера не должен обнулять остальной перевод: ключ,
    // которого в нём нет, обязан прийти из встроенного русского, а не из
    // английского фолбэка.
    assert_eq!(t("profile-sign-out"), "Выйти");
    // И только отсутствующий везде ключ доходит до английского.
    assert!(install_catalog(Locale::Ru, "x = y"));
    assert_eq!(t("profile-sign-out"), "Выйти");

    set_locale(Locale::En);
    assert_eq!(t("game-no-servers"), "No servers");
}
