//! The frame format on the wire. The other end is Java code that ships with a
//! build and stays on people's machines longer than this launcher version, so
//! the shape is pinned by tests instead of being whatever serde emits today.

use crate::{ToLauncher, ToMod, PROTOCOL};
use serde_json::json;

/// A frame with no fields is `{"type": …}` and no `data` at all. Half the mod's
/// intents look like this, so the Java side has to cope with it.
#[test]
fn unit_frame_has_no_data() {
    let text = serde_json::to_value(ToLauncher::CloseCase).unwrap();
    assert_eq!(text, json!({ "type": "CloseCase" }));
}

/// The mod updates on its own schedule, so `{"type": "RequestQueue"}` with no
/// `data` has to keep meaning "first page, no filter".
///
/// This is why the search fields went into a separate frame: adding them here
/// would give the frame a `data`, serde would refuse the version without one,
/// and every panel on an older build would quietly stop getting the queue.
#[test]
fn old_queue_request_without_data_parses() {
    let frame: ToLauncher = serde_json::from_value(json!({ "type": "RequestQueue" })).unwrap();
    assert!(matches!(frame, ToLauncher::RequestQueue));
}

#[test]
fn queue_page_carries_query_and_offset() {
    let frame: ToLauncher = serde_json::from_value(json!({
        "type": "RequestQueuePage",
        "data": { "query": "Steve", "offset": 20 }
    }))
    .unwrap();
    let ToLauncher::RequestQueuePage { query, offset } = frame else {
        panic!("not a RequestQueuePage");
    };
    assert_eq!(query.as_deref(), Some("Steve"));
    assert_eq!(offset, 20);
}

/// The other direction happens too: the build is newer than the launcher. An
/// empty `data` has to read as "first page, no filter" rather than fail.
#[test]
fn queue_page_with_empty_data_parses() {
    let frame: ToLauncher =
        serde_json::from_value(json!({ "type": "RequestQueuePage", "data": {} })).unwrap();
    let ToLauncher::RequestQueuePage { query, offset } = frame else {
        panic!("not a RequestQueuePage");
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

/// A field added on our side must not break a frame sent by an older mod.
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
        panic!("parsed into the wrong frame");
    };
    assert_eq!(resolution, "");
    assert_eq!(rule_code, None);
}

/// An unknown frame is a parse error, not a panic — the caller logs it and
/// keeps the connection.
#[test]
fn unknown_frame_is_an_error_not_a_panic() {
    let text = json!({ "type": "SomethingFromTheFuture", "data": {} });
    assert!(serde_json::from_value::<ToLauncher>(text).is_err());
}

/// The box around the card is a Rust detail; it must not show up on the wire.
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

/// The master sends the card under `case`, not `brief`. The launcher is what
/// parses it, so a rename on the master has to break here.
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

/// The rules come from the master in the master's shape: a section is `name`
/// there, a rule is `title`. Without the alias the whole response fails to
/// parse and the mod reports that the launcher can't reach the master — a shape
/// problem that looks like a dropped connection.
#[test]
fn rule_category_reads_masters_name() {
    let text = json!({ "id": uuid::Uuid::nil(), "name": "Chat", "sort_order": 1 });
    let category: crate::RuleCategory = serde_json::from_value(text).unwrap();
    assert_eq!(category.title, "Chat");
}

/// The master sends about three times the fields the mod needs.
#[test]
fn extra_master_fields_are_ignored() {
    let text = json!({
        "id": uuid::Uuid::nil(),
        "rule_id": uuid::Uuid::nil(),
        "kind": "mute",
        "label": "spam",
        "min_minutes": 15,
        "max_minutes": 120,
        "sort_order": 0,
    });
    let sanction: crate::RuleSanction = serde_json::from_value(text).unwrap();
    assert_eq!(sanction.kind, "mute");
    assert_eq!(sanction.min_minutes, Some(15));
}
