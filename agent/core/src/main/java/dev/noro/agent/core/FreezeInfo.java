package dev.noro.agent.core;

/**
 * Сведения о заморозке игрока.
 */
public record FreezeInfo(
        String reason,
        String frozenBy,
        String frozenAt) {
}
