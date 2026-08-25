use super::*;

fn errors(f: impl FnOnce(&mut Validation)) -> Vec<FieldError> {
    let mut v = Validation::new();
    f(&mut v);
    match v.finish() {
        Ok(()) => Vec::new(),
        Err(AppError::Validation(errors)) => errors,
        Err(other) => panic!("не Validation: {other:?}"),
    }
}

#[test]
fn чистая_форма_проходит() {
    let found = errors(|v| {
        v.required("name", "moderator")
            .max_len("name", "moderator", 32);
    });
    assert!(found.is_empty());
}

/// Три неверных поля — три подсказки за один заход, а не по одной на отправку.
#[test]
fn промахи_копятся_а_не_обрываются_на_первом() {
    let found = errors(|v| {
        v.required("name", "   ");
        v.one_of("kind", "explode", &["ban", "warn"]);
        v.positive("minutes", Some(-5));
    });
    assert_eq!(found.len(), 3);
    assert_eq!(found[0].field, "name");
    assert_eq!(found[0].code, "required");
    assert_eq!(found[1].code, "not_allowed");
    assert_eq!(found[2].code, "out_of_range");
}

/// Список допустимых значений попадает в текст: без него «not_allowed» не
/// говорит, что же можно.
#[test]
fn one_of_перечисляет_допустимое() {
    let found = errors(|v| {
        v.one_of("action", "nuke", &["delete", "flag"]);
    });
    assert!(
        found[0].message.contains("delete, flag"),
        "{}",
        found[0].message
    );
}

/// Длина считается символами: девять кириллических букв — это девять, а не
/// восемнадцать байт, иначе граница срабатывала бы вдвое раньше.
#[test]
fn длина_в_символах_а_не_в_байтах() {
    assert!(errors(|v| {
        v.max_len("display_name", "Модератор", 9);
    })
    .is_empty());
    assert_eq!(
        errors(|v| {
            v.max_len("display_name", "Модератор", 8);
        })
        .len(),
        1
    );
}

#[test]
fn пустое_значение_не_обязано_быть_положительным() {
    assert!(errors(|v| {
        v.positive("minutes", None);
    })
    .is_empty());
}
