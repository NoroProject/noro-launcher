package dev.noro.client.player;

/**
 * Игрокский мод → лаунчер.
 *
 * <p>Ни свод правил, ни свои наказания не требуют прав: свод публичен намеренно
 * — на него ссылается каждый бан, — а наказания игрок и так видит в кабинете.
 * Мод переносит это в игру, где вопрос и возникает.
 */
public sealed interface PlayerIntents {

    record RequestRules() implements PlayerIntents {}

    record RequestOwnPunishments() implements PlayerIntents {}
}
