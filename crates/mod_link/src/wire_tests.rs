//! Формат кадров на проводе. Его реализует чужой код на Java, который уезжает
//! со сборкой и живёт у людей дольше этой версии лаунчера, — поэтому форма
//! закреплена тестом, а не «как сериализуется, так и хорошо».

use crate::{ToLauncher, ToMod, PROTOCOL};
use serde_json::json;

/// Кадр без полей — это `{"type": …}` без `data`. Разбор на той стороне обязан
/// это пережить: половина намерений мода именно такие.
#[test]
fn unit_frame_has_no_data() {
    let text = serde_json::to_value(ToLauncher::CloseCase).unwrap();
    assert_eq!(text, json!({ "type": "CloseCase" }));
}

/// Мод уезжает со сборкой и обновляется отдельно от лаунчера, поэтому старый
/// `{"type": "RequestQueue"}` без `data` обязан и дальше означать «первая
/// страница, без фильтра».
///
/// Именно поэтому поля поиска уехали в отдельный кадр, а не в этот: у кадра без
/// полей нет `data`, и serde на его отсутствии останавливается — панель разбора
/// у всех, кто не обновил сборку, молча перестала бы получать очередь.
#[test]
fn старый_запрос_очереди_без_data_разбирается() {
    let frame: ToLauncher = serde_json::from_value(json!({ "type": "RequestQueue" })).unwrap();
    assert!(matches!(frame, ToLauncher::RequestQueue));
}

#[test]
fn страница_очереди_несёт_фильтр_и_сдвиг() {
    let frame: ToLauncher = serde_json::from_value(json!({
        "type": "RequestQueuePage",
        "data": { "query": "Steve", "offset": 20 }
    }))
    .unwrap();
    let ToLauncher::RequestQueuePage { query, offset } = frame else {
        panic!("не RequestQueuePage");
    };
    assert_eq!(query.as_deref(), Some("Steve"));
    assert_eq!(offset, 20);
}

/// Новый мод против старого лаунчера тоже бывает: сборку обновили, лаунчер ещё
/// нет. Незнакомый кадр там логируется и пропускается, а не роняет канал.
#[test]
fn страница_очереди_без_data_тоже_разбирается() {
    let frame: ToLauncher =
        serde_json::from_value(json!({ "type": "RequestQueuePage", "data": {} })).unwrap();
    let ToLauncher::RequestQueuePage { query, offset } = frame else {
        panic!("не RequestQueuePage");
    };
    assert_eq!(query, None);
    assert_eq!(offset, 0);
}

#[test]
fn hello_carries_key_and_protocol() {
    let frame = ToLauncher::Hello {
        key: "abc".into(),
        protocol: PROTOCOL,
    };
    assert_eq!(
        serde_json::to_value(frame).unwrap(),
        json!({ "type": "Hello", "data": { "key": "abc", "protocol": PROTOCOL } })
    );
}

/// Новое поле не должно ронять разбор кадра, пришедшего от старого мода:
/// ровно тем же приёмом держится `ClientWsMsg` при выпуске новых полей.
#[test]
fn missing_optional_fields_are_defaulted() {
    let text = json!({
        "type": "Resolve",
        "data": { "case_id": uuid::Uuid::nil(), "verdict": "confirmed" }
    });
    let frame: ToLauncher = serde_json::from_value(text).unwrap();
    let ToLauncher::Resolve {
        resolution,
        rule_code,
        ..
    } = frame
    else {
        panic!("разобралось не в тот кадр");
    };
    assert_eq!(resolution, "");
    assert_eq!(rule_code, None);
}

/// Незнакомый кадр — ошибка разбора, а не паника: соединение из-за него не
/// рвётся, кадр логируется и пропускается.
#[test]
fn unknown_frame_is_an_error_not_a_panic() {
    let text = json!({ "type": "SomethingFromTheFuture", "data": {} });
    assert!(serde_json::from_value::<ToLauncher>(text).is_err());
}

/// Бокс вокруг карточки — деталь Rust: на проводе его быть не должно.
#[test]
fn boxed_case_is_transparent_on_the_wire() {
    let view = crate::CaseView {
        brief: brief(),
        reports: Vec::new(),
        events: Vec::new(),
        punishments: Vec::new(),
        messages: Vec::new(),
        chat_allowed: true,
        reporters: Default::default(),
    };
    let text = serde_json::to_value(ToMod::Case {
        view: Box::new(view),
    })
    .unwrap();
    assert_eq!(text["type"], "Case");
    assert_eq!(text["data"]["view"]["case"]["number"], 7);
}

/// Карточка приходит с мастера под ключом `case`, а не `brief`: разбирает её
/// лаунчер, и переименование поля на мастере обязано ломаться здесь.
#[test]
fn card_is_read_from_the_master_shape() {
    let text = json!({
        "case": serde_json::to_value(brief()).unwrap(),
        "chat_allowed": false,
    });
    let view: crate::CaseView = serde_json::from_value(text).unwrap();
    assert_eq!(view.brief.number, 7);
    assert!(view.events.is_empty());
}

fn brief() -> crate::CaseBrief {
    crate::CaseBrief {
        id: uuid::Uuid::nil(),
        number: 7,
        target_id: uuid::Uuid::nil(),
        target_name: Some("target".into()),
        game_server_id: None,
        server_name: None,
        status: "open".into(),
        claimed_by: None,
        claimed_by_name: None,
        opened_at: chrono::Utc::now(),
        resolved_at: None,
        verdict: None,
        rule_code: None,
        reports_count: 1,
        reporters_count: 1,
        last_report_at: None,
    }
}

/// Свод правил приходит с мастера, и его форма — не наша.
///
/// Раздел там зовётся `name`, пункт — `title`. Без псевдонима разбор всего
/// ответа падал целиком, а мод показывал «лаунчер не достучался до мастера»:
/// ошибка была в форме, а выглядела как обрыв связи.
#[test]
fn rule_category_reads_masters_name() {
    let text = json!({ "id": uuid::Uuid::nil(), "name": "Чат", "sort_order": 1 });
    let category: crate::RuleCategory = serde_json::from_value(text).unwrap();
    assert_eq!(category.title, "Чат");
}

/// Лишние поля мастера не должны мешать: их там втрое больше, чем нужно моду.
#[test]
fn extra_master_fields_are_ignored() {
    let text = json!({
        "id": uuid::Uuid::nil(),
        "rule_id": uuid::Uuid::nil(),
        "kind": "mute",
        "label": "за флуд",
        "min_minutes": 15,
        "max_minutes": 120,
        "sort_order": 0,
    });
    let sanction: crate::RuleSanction = serde_json::from_value(text).unwrap();
    assert_eq!(sanction.kind, "mute");
    assert_eq!(sanction.min_minutes, Some(15));
}
