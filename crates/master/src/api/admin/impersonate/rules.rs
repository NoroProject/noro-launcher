//! Who may be impersonated.
//!
//! An impersonated session carries everything the target can do, so this check
//! is the only thing standing between the impersonate permission and full
//! privilege escalation. It runs on the server for every grant.

use schema::{permission_matches, UserProfile};

/// Whether the actor's permissions cover the target's.
///
/// Coverage uses the same wildcards as a normal permission check — an actor
/// holding `noro.admin.*` covers a target's `noro.admin.users`. Matching
/// literally would deny nearly everything: players are granted by name, admins
/// by pattern.
pub fn can_impersonate(actor: &UserProfile, target: &UserProfile) -> bool {
    // Impersonating yourself would just hide your own actions behind an
    // impersonation record.
    if actor.id == target.id {
        return false;
    }
    // Root is the operator account, so it covers everything by definition.
    if target.is_root {
        return false;
    }

    target.all_permissions().all(|needed| {
        actor
            .all_permissions()
            .any(|held| permission_matches(held, needed))
    })
}

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;
