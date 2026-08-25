//! Реестр OAuth2-scope'ов: что стороннее приложение вправе спросить у мастера.
//!
//! Заводятся только здесь — ручки принимают константу отсюда, поэтому проверить
//! незарегистрированный scope нельзя, не сломав сборку. Список отдаётся сайту:
//! и экран согласия, и админка показывают одни и те же формулировки.

/// Кому scope доступен.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeTier {
    /// Может запросить любое одобренное приложение.
    Basic,
    /// Только если оператор выдал его приложению отдельно: за этими данными
    /// либо чужая приватность, либо право что-то менять.
    Privileged,
    /// Наш лаунчер и сайт. Сторонним не выдаётся никогда — это полный доступ
    /// к аккаунту, а не доступ к отдельным данным.
    Internal,
}

/// Scope: имя, уровень доступа и человеческая формулировка.
///
/// `title` — запасной текст. Экран согласия сначала ищет перевод по ключу
/// `oauth-scope-<slug>`, потому что читает его игрок, а не разработчик.
pub struct Scope {
    pub name: &'static str,
    pub tier: ScopeTier,
    pub title: &'static str,
}

macro_rules! scopes {
    ($($konst:ident = $name:literal, $tier:expr, $title:literal;)*) => {
        $(pub const $konst: &str = $name;)*

        pub const ALL_SCOPES: &[&Scope] = &[$(&Scope {
            name: $name, tier: $tier, title: $title,
        }),*];
    };
}

scopes! {
    // --- Доступно любому приложению ------------------------------------------
    SCOPE_IDENTITY    = "identity",    ScopeTier::Basic, "Your nickname, UUID and avatar";
    SCOPE_PROFILE     = "profile",     ScopeTier::Basic, "Your roles and account status";
    SCOPE_SKINS       = "skins",       ScopeTier::Basic, "Your skin and cape";
    SCOPE_CAPES       = "capes",       ScopeTier::Basic, "Which capes you may wear";
    SCOPE_PUNISHMENTS = "punishments", ScopeTier::Basic, "Your active punishments";
    SCOPE_SERVERS     = "servers",     ScopeTier::Basic, "The project's server list and online count";

    // --- Только по решению оператора -----------------------------------------
    SCOPE_SKINS_WRITE = "skins:write", ScopeTier::Privileged, "Change your skin and cape";
    SCOPE_IDENTITIES  = "identities",  ScopeTier::Privileged, "See which platforms you sign in with";
    SCOPE_JOURNAL     = "journal",     ScopeTier::Privileged, "Your launch history: builds and mods";

    // --- Наши собственные ----------------------------------------------------
    SCOPE_LAUNCHER    = "launcher",    ScopeTier::Internal, "Full account access from the launcher";
    SCOPE_SITE        = "site",        ScopeTier::Internal, "Full account access from the site";
    SCOPE_MASTER      = "master",      ScopeTier::Internal, "Full account access";
}

/// Scope'ы, за которыми стоит полный доступ к аккаунту.
///
/// Сессия с таким scope — это наш собственный вход (сайт, лаунчер, восстановление
/// доступа). Всё остальное — токен приложения, и такой токен пускают только в
/// `/api/oauth/*`, где каждая ручка проверяет свой scope.
pub fn is_internal(scope: &str) -> bool {
    ALL_SCOPES
        .iter()
        .any(|s| s.name == scope && s.tier == ScopeTier::Internal)
}

/// Разбор строки scope'ов запроса: пробелы, как в OAuth 2.0, плюс запятые —
/// их подставляют почти все, кто пишет интеграцию руками.
pub fn parse_scopes(raw: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for part in raw.split([' ', ',', '+']) {
        let part = part.trim();
        if !part.is_empty() && !out.iter().any(|s| s == part) {
            out.push(part.to_string());
        }
    }
    out
}

/// Известен ли scope и можно ли его вообще выдать приложению.
pub fn grantable(scope: &str) -> Option<&'static Scope> {
    ALL_SCOPES
        .iter()
        .copied()
        .find(|s| s.name == scope && s.tier != ScopeTier::Internal)
}

/// Scope'ы, которые приложение вправе просить без отдельного решения оператора.
pub fn default_allowed() -> Vec<String> {
    ALL_SCOPES
        .iter()
        .filter(|s| s.tier == ScopeTier::Basic)
        .map(|s| s.name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_scopes_are_not_grantable() {
        assert!(grantable(SCOPE_LAUNCHER).is_none());
        assert!(grantable(SCOPE_SITE).is_none());
        assert!(grantable(SCOPE_IDENTITY).is_some());
    }

    #[test]
    fn our_own_sessions_are_recognised() {
        assert!(is_internal("launcher"));
        assert!(is_internal("site"));
        assert!(!is_internal("identity"));
        assert!(!is_internal("identity profile"));
    }

    #[test]
    fn scope_strings_split_on_spaces_and_commas() {
        assert_eq!(parse_scopes("identity profile"), ["identity", "profile"]);
        assert_eq!(parse_scopes("identity, profile"), ["identity", "profile"]);
        assert_eq!(parse_scopes("identity identity"), ["identity"]);
        assert!(parse_scopes("   ").is_empty());
    }

    #[test]
    fn basic_scopes_are_the_default_grant() {
        let allowed = default_allowed();
        assert!(allowed.iter().any(|s| s == SCOPE_IDENTITY));
        assert!(!allowed.iter().any(|s| s == SCOPE_SKINS_WRITE));
    }
}
