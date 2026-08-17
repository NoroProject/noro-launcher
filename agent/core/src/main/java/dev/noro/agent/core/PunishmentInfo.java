package dev.noro.agent.core;

import java.time.Instant;
import java.util.UUID;

/**
 * Наказание в том виде, в каком его показывают игроку.
 *
 * <p>Один и тот же тип приходит тремя дорогами: в профиле на входе, ответом на
 * команду и кадром живого канала. Все три обязаны рисовать одинаковый экран —
 * поэтому и модель одна.
 */
public record PunishmentInfo(
        UUID id,
        /** {@code ban} | {@code server_ban} | {@code mute} | {@code warn}. */
        String kind,
        String reason,
        /** Ник модератора либо {@code Agent: <сервер>}, если наказал сервер. */
        String actorLabel,
        Instant createdAt,
        /** {@code null} — навсегда. */
        Instant expiresAt,
        /** Когда сняли. {@code null} — действует. Живой канал это поле не шлёт. */
        Instant revokedAt,
        /** Код правила из свода. {@code null} — правило не указывали. */
        String ruleCode) {

    public boolean permanent() {
        return expiresAt == null;
    }

    /** Действует ли прямо сейчас: не снято и срок не вышел. */
    public boolean active() {
        return revokedAt == null && (expiresAt == null || expiresAt.isAfter(Instant.now()));
    }

    /**
     * Сколько осталось. Для вечного — {@code null}.
     *
     * <p>Возвращается длительность, а не минуты: на последней минуте «ещё 1
     * минута» висит не меняясь, и мут выглядит заглючившим. Секунды видно.
     */
    public java.time.Duration left() {
        if (expiresAt == null) {
            return null;
        }
        java.time.Duration left = java.time.Duration.between(Instant.now(), expiresAt);
        return left.isNegative() ? java.time.Duration.ZERO : left;
    }

    /** Выкидывает ли это наказание с сервера. */
    public boolean disconnects() {
        return "ban".equals(kind) || "server_ban".equals(kind);
    }
}
