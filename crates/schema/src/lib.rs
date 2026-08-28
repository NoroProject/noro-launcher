//! Serde types shared by the launcher, master, CLI and — as JSON — the web app.
//!
//! Nothing here depends on tokio, axum or sqlx. That's what lets the GPUI
//! frontend, the backend and the master all pull it in.

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

/// Namespace for the v5 UUIDs below. There is no stored mapping from external
/// account to Minecraft UUID — it is derived every time, so this must not change.
pub const MC_UUID_NAMESPACE: uuid::Uuid = uuid::Uuid::from_bytes([
    0x4e, 0x6f, 0x72, 0x6f, 0x4d, 0x43, 0x55, 0x55, 0x49, 0x44, 0x4e, 0x53, 0x70, 0x61, 0x63, 0x65,
]);

/// Offline-style MC UUID for a linked account, e.g. `("discord", "123")`.
///
/// The provider name is capitalised because every existing player's UUID was
/// derived from `Discord:123…`. Changing the name scheme changes every UUID, and
/// a player who loses theirs loses inventory, progress and permissions on every
/// server at once.
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

pub fn mc_uuid_from_discord(discord_id: &str) -> uuid::Uuid {
    mc_uuid_from_identity("discord", discord_id)
}

pub fn mc_uuid_from_telegram(telegram_id: &str) -> uuid::Uuid {
    mc_uuid_from_identity("telegram", telegram_id)
}

/// Dev-only ed25519 seed: both sides derive their half of the manifest signing
/// pair from it, so a local master and a local launcher agree out of the box.
///
/// Production passes the real key through `NORO_SIGNING_KEY` (master) and
/// `NORO_SIGNING_PUBKEY` (launcher, at build time). This seed is in the sources
/// for anyone to read — it must never reach prod.
pub const DEV_SIGNING_SEED: [u8; 32] = *b"noro-launcher-dev-signing-seed!!";

/// Name of the launcher's directory inside the system data dir.
///
/// Debug builds get their own, or development on a machine that also has the
/// launcher installed would overwrite the real `config.json`, instances and
/// downloaded core, and point them at a local master signing with the dev key.
/// `NORO_LAUNCHER_DIR` overrides it when a third isolated profile is needed.
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

    /// Pins the value, since nothing in the database records what a player's
    /// UUID used to be — it is re-derived on every login.
    #[test]
    fn discord_id_maps_to_a_stable_uuid() {
        let id = "123456789012345678";
        assert_eq!(
            mc_uuid_from_discord(id).to_string(),
            mc_uuid_from_discord(id).to_string(),
            "same input must give the same UUID"
        );
        assert_eq!(
            mc_uuid_from_discord(id),
            uuid::Uuid::new_v5(&MC_UUID_NAMESPACE, b"Discord:123456789012345678"),
            "name scheme changed — every existing player gets a new UUID"
        );
    }

    /// The same numeric id on Discord and on Twitch is a coincidence, not one
    /// player.
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
