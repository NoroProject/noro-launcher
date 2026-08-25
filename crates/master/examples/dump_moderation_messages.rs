//! Дефолтные тексты наказаний в JSON — для заливки в `instance_settings`.
fn main() {
    let map = serde_json::json!({
        "ru": master::api::moderation_messages::ModerationMessages::default_ru(),
        "en": master::api::moderation_messages::ModerationMessages::default_en(),
    });
    println!("{}", serde_json::to_string(&map).unwrap());
}
