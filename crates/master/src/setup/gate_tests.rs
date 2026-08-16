//! Список открытых путей — то, что нельзя расширить случайно: каждый лишний
//! путь работает на ненастроенном инстансе без всякой аутентификации.

use super::*;

#[test]
fn health_stays_open_so_the_orchestrator_does_not_loop() {
    assert!(always_open("/health"));
}

#[test]
fn the_wizard_is_open() {
    assert!(always_open("/api/setup/status"));
    assert!(always_open("/api/setup/complete"));
}

#[test]
fn everything_else_is_closed() {
    for path in [
        "/api/me",
        "/ws/launcher",
        "/api/admin/users",
        "/api/yggdrasil/",
        "/auth/discord/login",
        "/files/abc",
    ] {
        assert!(!always_open(path), "{path} не должен быть открыт");
    }
}

#[test]
fn a_lookalike_prefix_does_not_open_anything() {
    // `/api/setupx` начинается так же, но визардом не является.
    assert!(!always_open("/api/setupx/steal"));
    assert!(!always_open("/api/setup"));
    assert!(!always_open("/healthz"));
}
