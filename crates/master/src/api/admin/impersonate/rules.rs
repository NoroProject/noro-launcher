//! Кого можно «надеть».
//!
//! При выбранной модели «под impersonation доступно всё, что доступно игроку»
//! это единственная преграда против эскалации привилегий: без неё админ с
//! правом impersonate входит в аккаунт другого админа и получает его права.
//! Поэтому правило жёсткое и проверяется на сервере при каждом гранте.

use schema::{UserProfile, permission_matches};

/// Строго ли права цели покрываются правами актора.
///
/// «Покрываются» — по тем же шаблонам, что и обычная проверка: `noro.admin.*`
/// у актора покрывает `noro.admin.users` у цели. Иначе правило запрещало бы
/// почти всё: у игроков права выданы поимённо, у админов — шаблонами.
pub fn can_impersonate(actor: &UserProfile, target: &UserProfile) -> bool {
    // Себя надевать незачем, и это скрыло бы действия админа под видом
    // impersonation.
    if actor.id == target.id {
        return false;
    }
    // Root — операторский аккаунт с полным доступом; «надеть» его значит
    // получить всё, что есть в системе.
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
