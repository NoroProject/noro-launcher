package dev.noro.agent.core;

import com.google.gson.annotations.SerializedName;
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
        String reason,
        // Мастер зовёт это поле `case`: `rename_all = "snake_case"` у него
        // переименовывает варианты, а не поля. Без явного имени Gson искал бы
        // `case_id`, получал null — и разбор молча не работал бы весь.
        @SerializedName("case") UUID caseId,
        UUID moderator,
        UUID reporter,
        String reporterName,
        String world,
        Double x,
        Double y,
        Double z,
        Boolean closed,
        Integer beforeSecs) {

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
            case "case_assigned":
                listener.onCaseAssigned(new CaseSession(
                        caseId, target, targetName, reporter, reporterName,
                        reason == null ? "" : reason, world, x, y, z, null, false), moderator);
                return;
            case "case_finished":
                listener.onCaseFinished(moderator, caseId, Boolean.TRUE.equals(closed));
                return;
            case "case_chat_request":
                listener.onCaseChatRequest(caseId, beforeSecs == null ? 0 : beforeSecs);
                return;
            case "case_inventory_request":
                listener.onCaseInventoryRequest(caseId, target);
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
