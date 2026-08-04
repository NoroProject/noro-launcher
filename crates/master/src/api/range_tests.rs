//! Разбор Range: границы, которые легко сломать и трудно заметить в проде.

use super::*;

fn part(raw: &str, total: u64) -> (u64, u64) {
    match parse_range(raw, total) {
        Wanted::Part { start, end } => (start, end),
        Wanted::Whole => panic!("{raw}: ожидали диапазон, получили whole"),
        Wanted::Unsatisfiable => panic!("{raw}: ожидали диапазон, получили 416"),
    }
}

fn is_unsatisfiable(raw: &str, total: u64) -> bool {
    matches!(parse_range(raw, total), Wanted::Unsatisfiable)
}

fn is_whole(raw: &str, total: u64) -> bool {
    matches!(parse_range(raw, total), Wanted::Whole)
}

#[test]
fn open_ended_range_runs_to_the_last_byte() {
    // Форма докачки лаунчера: «есть первые 100 байт, дай остальное».
    assert_eq!(part("bytes=100-", 1000), (100, 999));
    assert_eq!(part("bytes=0-", 1000), (0, 999));
}

#[test]
fn closed_range_is_inclusive_and_clamped() {
    assert_eq!(part("bytes=0-99", 1000), (0, 99));
    // Конец за пределами файла подрезается, а не превращается в 416.
    assert_eq!(part("bytes=900-5000", 1000), (900, 999));
    assert_eq!(part("bytes=999-999", 1000), (999, 999));
}

#[test]
fn suffix_range_counts_from_the_end() {
    assert_eq!(part("bytes=-200", 1000), (800, 999));
    // Суффикс длиннее файла — весь файл, но всё ещё 206.
    assert_eq!(part("bytes=-5000", 1000), (0, 999));
}

#[test]
fn out_of_bounds_is_rejected() {
    // Именно этот случай отделяет корректную докачку от бесконечного цикла:
    // файл уже целиком на диске, лаунчер просит с конца.
    assert!(is_unsatisfiable("bytes=1000-", 1000));
    assert!(is_unsatisfiable("bytes=2000-3000", 1000));
    assert!(is_unsatisfiable("bytes=-0", 1000));
    assert!(is_unsatisfiable("bytes=500-100", 1000));
    assert!(is_unsatisfiable("bytes=abc-", 1000));
    assert!(is_unsatisfiable("bytes=0-", 0));
}

#[test]
fn unsupported_forms_fall_back_to_the_whole_file() {
    // Мультидиапазоны и чужие единицы измерения — отдаём 200, это законно.
    assert!(is_whole("bytes=0-99,200-299", 1000));
    assert!(is_whole("items=0-99", 1000));
    assert!(is_whole("garbage", 1000));
}
