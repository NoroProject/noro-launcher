package dev.noro.agent.core;

import java.time.Instant;
import java.util.UUID;

/**
 * Кадр живого канала.
 */
record LinkFrame(
        String type,
        LivePunishment punishment,
        UUID id,
        UUID uuid,
        UUID target,
        String targetName,
        String kind,
        String actorLabel,
        String message,
        Integer countdownSeconds,
        String reason) {

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
            case "filters_changed":
                listener.onFiltersChanged();
                return;
            case "restart_notice":
                int sec = countdownSeconds != null ? countdownSeconds : 60;
                listener.onRestartNotice(sec, reason == null ? "Planned restart" : reason);
                return;
            case "profile_changed":
                listener.onProfileChanged(uuid);
                return;
            case "kick":
                listener.onKick(target, message);
                return;
            case "tell":
                listener.onTell(target, message);
                return;
            case "announce":
                listener.onAnnounce(message);
                return;
            case "maintenance_start":
                listener.onMaintenanceStart(countdownSeconds != null ? countdownSeconds : 60, reason);
                return;
            case "maintenance_cancel":
                listener.onMaintenanceCancel();
                return;
            default:
                throw new IllegalArgumentException("unknown frame type " + type);
        }
    }

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
