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

    /** Сколько осталось в минутах. Для вечного — {@code -1}. */
    public long minutesLeft() {
        if (expiresAt == null) {
            return -1;
        }
        long left = java.time.Duration.between(Instant.now(), expiresAt).toMinutes();
        // Меньше минуты — это всё ещё «одна минута», а не «ноль»: ноль в тексте
        // читается как «уже свободен», хотя игрок ещё нет.
        return Math.max(left, 1);
    }

    /** Выкидывает ли это наказание с сервера. */
    public boolean disconnects() {
        return "ban".equals(kind) || "server_ban".equals(kind);
    }
}
