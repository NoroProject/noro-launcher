package dev.noro.agent.core;

import java.time.Instant;
import java.util.UUID;

/**
 * Кадр живого канала. Плоская модель под все три вида: у Gson нет разбора по
 * полю-тегу, а заводить ради трёх кадров иерархию с адаптером — больше кода,
 * чем самого протокола.
 *
 * <p>Поля должны совпадать с {@code ToAgent} в
 * {@code crates/master/src/agent_link/proto.rs}.
 */
record LinkFrame(
        String type,
        LivePunishment punishment,
        UUID id,
        UUID target,
        String targetName,
        String kind,
        String actorLabel) {

    void deliver(AgentLink.Listener listener) {
        switch (type == null ? "" : type) {
            case "punished":
                if (punishment != null) {
                    listener.onPunished(punishment.target(), punishment.targetName(), punishment.toInfo());
                }
                return;
            case "revoked":
                listener.onRevoked(target, targetName, kind, actorLabel);
                return;
            case "messages_changed":
                listener.onMessagesChanged();
                return;
            default:
                throw new IllegalArgumentException("unknown frame type " + type);
        }
    }

    /** Наказание с адресатом: в канале игрок назван, в самом наказании — нет. */
    record LivePunishment(
            UUID id,
            UUID target,
            String targetName,
            String kind,
            String reason,
            String actorLabel,
            Instant createdAt,
            Instant expiresAt,
            String ruleCode) {

        PunishmentInfo toInfo() {
            return new PunishmentInfo(id, kind, reason, actorLabel, createdAt, expiresAt, null, ruleCode);
        }
    }
}
