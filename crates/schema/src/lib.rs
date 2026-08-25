//! Общие serde-типы, разделяемые между лаунчером, мастером, CLI и (через JSON) web.
//!
//! Этот крейт не зависит от tokio/axum/sqlx — только сериализация и базовые типы,
//! поэтому его одинаково тянут и frontend (GPUI), и backend, и master.

pub mod admin_ws;
pub mod blocklist;
pub mod build;
pub mod integrity;
pub mod launcher;
pub mod manifest_args;
pub mod news;
pub mod optional;
pub mod page;
pub mod path_rules;
pub mod permissions;
pub mod redact;
pub mod scopes;
pub mod server;
pub mod user;
pub mod ws_protocol;

pub use admin_ws::*;
pub use blocklist::*;
pub use build::*;
pub use integrity::*;
pub use launcher::*;
pub use manifest_args::*;
pub use news::*;
pub use page::Page;
pub use path_rules::*;
pub use permissions::*;
pub use redact::redact;
pub use scopes::*;
pub use server::*;
pub use user::*;
pub use ws_protocol::*;

/// UUID игрока (Minecraft) детерминированно выводится из привязки к платформе
/// через UUID v5 в фиксированном namespace. Так один и тот же внешний аккаунт
/// всегда получает один и тот же MC-UUID, без хранения маппинга.
pub const MC_UUID_NAMESPACE: uuid::Uuid = uuid::Uuid::from_bytes([
    0x4e, 0x6f, 0x72, 0x6f, 0x4d, 0x43, 0x55, 0x55, 0x49, 0x44, 0x4e, 0x53, 0x70, 0x61, 0x63, 0x65,
]);

/// Детерминированный offline-style MC UUID из привязки: `("discord", "123")`.
///
/// Имя платформы идёт с заглавной буквы — именно так («Discord:123…») выведены
/// UUID всех существующих игроков. Схему трогать нельзя: вместе с UUID игрок
/// теряет инвентарь, прогресс и права на всех серверах сразу.
pub fn mc_uuid_from_identity(provider: &str, provider_user_id: &str) -> uuid::Uuid {
    let mut name = String::with_capacity(provider.len() + provider_user_id.len() + 1);
    let mut chars = provider.chars();
    if let Some(first) = chars.next() {
        name.extend(first.to_uppercase());
        name.push_str(chars.as_str());
    }
    name.push(':');
    name.push_str(provider_user_id);
    uuid::Uuid::new_v5(&MC_UUID_NAMESPACE, name.as_bytes())
}

/// Детерминированный offline-style MC UUID из Discord ID.
pub fn mc_uuid_from_discord(discord_id: &str) -> uuid::Uuid {
    mc_uuid_from_identity("discord", discord_id)
}

/// Детерминированный offline-style MC UUID из Telegram ID.
pub fn mc_uuid_from_telegram(telegram_id: &str) -> uuid::Uuid {
    mc_uuid_from_identity("telegram", telegram_id)
}

/// DEV-ONLY seed для ed25519. В режиме разработки мастер выводит из него приватный
/// ключ подписи манифестов, а лаунчер — публичный ключ для проверки. Так обе
/// стороны согласованы из коробки. В production мастер задаёт реальный приватный
/// ключ через env `NORO_SIGNING_KEY`, а лаунчер компилируется с реальным публичным
/// ключом через env `NORO_SIGNING_PUBKEY`. Этот seed НИКОГДА не должен использоваться
/// в проде — он публичен в исходниках.
pub const DEV_SIGNING_SEED: [u8; 32] = *b"noro-launcher-dev-signing-seed!!";

/// Имя каталога данных лаунчера внутри системного data-dir.
///
/// Debug-сборка живёт отдельно от установленной: иначе разработка затирает
/// боевой `config.json`, инстансы и скачанный core на той же машине, а лаунчер
/// начинает ходить в локальный мастер, чьи манифесты подписаны dev-ключом.
/// `NORO_LAUNCHER_DIR` перекрывает выбор, когда нужен ещё один изолированный
/// профиль.
pub fn launcher_dir_name() -> String {
    match std::env::var("NORO_LAUNCHER_DIR") {
        Ok(custom) if !custom.is_empty() => custom,
        _ if cfg!(debug_assertions) => "noro-launcher-dev".into(),
        _ => "noro-launcher".into(),
    }
}

#[cfg(test)]
mod uuid_tests {
    use super::*;

    /// Маппинга Discord → MC в базе нет: UUID выводится заново при каждом
    /// заходе. Если значение когда-нибудь поедет, игроки потеряют инвентарь,
    /// прогресс и права на всех серверах сразу — поэтому оно зафиксировано.
    #[test]
    fn discord_id_maps_to_a_stable_uuid() {
        let id = "123456789012345678";
        assert_eq!(
            mc_uuid_from_discord(id).to_string(),
            mc_uuid_from_discord(id).to_string(),
            "одинаковый вход обязан давать одинаковый UUID"
        );
        assert_eq!(
            mc_uuid_from_discord(id),
            uuid::Uuid::new_v5(&MC_UUID_NAMESPACE, b"Discord:123456789012345678"),
            "схема имени изменилась — все существующие игроки сменят UUID"
        );
    }

    /// Разные платформы не должны сталкиваться между собой: одинаковые
    /// числовые id у Discord и Twitch — обычное совпадение, а не один игрок.
    #[test]
    fn providers_do_not_collide() {
        assert_ne!(
            mc_uuid_from_identity("discord", "12345"),
            mc_uuid_from_identity("twitch", "12345")
        );
        assert_eq!(
            mc_uuid_from_identity("twitch", "12345"),
            uuid::Uuid::new_v5(&MC_UUID_NAMESPACE, b"Twitch:12345")
        );
    }

    #[test]
    fn different_accounts_get_different_uuids() {
        assert_ne!(
            mc_uuid_from_discord("111111111111111111"),
            mc_uuid_from_discord("222222222222222222")
        );
    }

    #[test]
    fn namespace_is_pinned() {
        assert_eq!(
            MC_UUID_NAMESPACE.to_string(),
            "4e6f726f-4d43-5555-4944-4e5370616365"
        );
    }
}
