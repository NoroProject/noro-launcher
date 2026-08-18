package dev.noro.agent.core;

/**
 * Тексты, которые игрок видит при бане, муте, предупреждении и техработах.
 */
public record MessageTemplates(
        String banPermanent,
        String banTemporary,
        String serverBanPermanent,
        String serverBanTemporary,
        String mutePermanent,
        String muteTemporary,
        String muteActionbarPermanent,
        String muteActionbarTemporary,
        String warnActionbar,
        String warnNotice,
        String broadcast,
        String actorReceipt,
        String reasonByRule,
        String noAccount,
        String noAccess,
        String maintenance,
        String rulesUrl) {

    public static MessageTemplates defaults() {
        return new MessageTemplates(
                "You are banned from this network.\n\nReason: {reason}\nBy: {actor}\nCase: {id}",
                "You are banned from this network.\n\nReason: {reason}\nExpires: {expires} (in {duration})\nBy: {actor}\nCase: {id}",
                "You are banned from this server.\n\nReason: {reason}\nBy: {actor}\nCase: {id}",
                "You are banned from this server.\n\nReason: {reason}\nExpires: {expires} (in {duration})\nBy: {actor}\nCase: {id}",
                "You are muted. Reason: {reason}",
                "You are muted for another {duration}. Reason: {reason}",
                "Muted • {reason}",
                "Muted for another {duration} • {reason}",
                "Warning • {reason}",
                "You have been warned by {actor}. Reason: {reason}",
                "{player} was {kind} by {actor}: {reason}",
                "{player} was {kind}: {reason}",
                "Rule violation, {rule}: {rule_title}",
                "No account on this network.\n\nSign in through the launcher first — it will create one and bring you back.",
                "You do not have access to this server.",
                "The server is currently under maintenance.",
                "");
    }

    public MessageTemplates complete() {
        MessageTemplates d = defaults();
        return new MessageTemplates(
                pick(banPermanent, d.banPermanent),
                pick(banTemporary, d.banTemporary),
                pick(serverBanPermanent, d.serverBanPermanent),
                pick(serverBanTemporary, d.serverBanTemporary),
                pick(mutePermanent, d.mutePermanent),
                pick(muteTemporary, d.muteTemporary),
                pick(muteActionbarPermanent, d.muteActionbarPermanent),
                pick(muteActionbarTemporary, d.muteActionbarTemporary),
                pick(warnActionbar, d.warnActionbar),
                pick(warnNotice, d.warnNotice),
                broadcast == null ? d.broadcast : broadcast,
                pick(actorReceipt, d.actorReceipt),
                pick(reasonByRule, d.reasonByRule),
                pick(noAccount, d.noAccount),
                pick(noAccess, d.noAccess),
                pick(maintenance, d.maintenance),
                rulesUrl == null ? "" : rulesUrl);
    }

    public String actionbar(PunishmentInfo punishment) {
        boolean permanent = punishment.permanent();
        if ("mute".equals(punishment.kind())) {
            return permanent ? muteActionbarPermanent : muteActionbarTemporary;
        }
        return "warn".equals(punishment.kind()) ? warnActionbar : screen(punishment);
    }

    public String screen(PunishmentInfo punishment) {
        boolean permanent = punishment.permanent();
        switch (punishment.kind()) {
            case "ban":
                return permanent ? banPermanent : banTemporary;
            case "server_ban":
                return permanent ? serverBanPermanent : serverBanTemporary;
            case "mute":
                return permanent ? mutePermanent : muteTemporary;
            default:
                return warnNotice;
        }
    }

    private static String pick(String value, String fallback) {
        return value == null || value.isBlank() ? fallback : value;
    }
}
