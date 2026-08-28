//! Таблица решений из плана §10.5, проверенная по клеткам. Ошибка здесь либо
//! затирает правки игрока, либо навсегда оставляет его на старом конфиге.

use super::*;

const MINE: &str = "aaa";
const BASE: &str = "bbb";
const THEIRS: &str = "ccc";

#[test]
fn untouched_by_the_player_gets_updated() {
    // Совпадает с базой, сервер изменил → обновить. Ровно тот случай, ради
    // которого режим и появился: user_managed оставлял игрока на старом навсегда.
    assert_eq!(decide(Some(BASE), Some(BASE), THEIRS), Decision::Update);
}

#[test]
fn edited_by_the_player_and_untouched_by_the_server_stays() {
    assert_eq!(decide(Some(MINE), Some(BASE), BASE), Decision::KeepMine);
}

#[test]
fn edited_by_both_is_a_conflict() {
    assert_eq!(decide(Some(MINE), Some(BASE), THEIRS), Decision::Conflict);
}

#[test]
fn nothing_changed_means_nothing_to_do() {
    assert_eq!(decide(Some(BASE), Some(BASE), BASE), Decision::Nothing);
}

#[test]
fn identical_edits_are_not_a_conflict() {
    // Обе стороны пришли к одному и тому же содержимому — спорить не о чем.
    assert_eq!(decide(Some(THEIRS), Some(BASE), THEIRS), Decision::Nothing);
}

#[test]
fn a_missing_file_is_always_installed() {
    // Файла нет — ставим, кто бы что ни менял.
    assert_eq!(decide(None, Some(BASE), THEIRS), Decision::Update);
    assert_eq!(decide(None, None, THEIRS), Decision::Update);
}

#[test]
fn without_a_base_a_difference_is_treated_as_a_conflict() {
    // Первый проход на уже установленной сборке: базы нет, и затирать чужой
    // файл только потому, что мы его не помним, нельзя.
    assert_eq!(decide(Some(MINE), None, THEIRS), Decision::Conflict);
    assert_eq!(decide(Some(THEIRS), None, THEIRS), Decision::Nothing);
}

#[tokio::test]
async fn base_hashes_survive_a_round_trip() {
    let dir = std::env::temp_dir().join(format!("noro-merge-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let mut base = BaseHashes::default();
    base.set("config/sodium.json", BASE);
    base.save(&dir).await;

    let loaded = BaseHashes::load(&dir).await;
    assert_eq!(loaded.get("config/sodium.json"), Some(BASE));
    assert_eq!(loaded.get("config/other.json"), None);

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn a_conflict_backup_keeps_the_players_version() {
    let dir = std::env::temp_dir().join(format!("noro-conflict-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("config")).unwrap();
    std::fs::write(dir.join("config/sodium.json"), "правки игрока").unwrap();

    backup_conflict(&dir, "config/sodium.json", "20260816-120000")
        .await
        .unwrap();

    let saved =
        std::fs::read_to_string(dir.join(".noro/conflicts/20260816-120000/config/sodium.json"))
            .unwrap();
    assert_eq!(saved, "правки игрока");

    let _ = std::fs::remove_dir_all(&dir);
}
