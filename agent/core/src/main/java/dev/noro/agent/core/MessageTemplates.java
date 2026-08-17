package dev.noro.agent.core;

/**
 * Тексты, которые игрок видит при бане, муте и предупреждении.
 *
 * <p>Приходят с мастера ({@code GET /api/agent/messages}) и правятся в админке.
 * Встроенные значения — не «дефолты на всякий случай», а рабочий набор: сервер
 * обязан внятно объяснить отказ даже когда мастер недоступен.
 *
 * <p>Поля совпадают с полями {@code ModerationMessages} на мастере: имена
 * переводит политика Gson, поэтому добавлять поле надо в обоих местах.
 */
public record MessageTemplates(
        String banPermanent,
        String banTemporary,
        String serverBanPermanent,
        String serverBanTemporary,
        String mutePermanent,
        String muteTemporary,
        String warnNotice,
        /** Пусто — не объявлять о наказании в чат. */
        String broadcast,
        String actorReceipt) {

    public static MessageTemplates defaults() {
        return new MessageTemplates(
                "You are banned from this network.\n\nReason: {reason}\nBy: {actor}\nCase: {id}",
                "You are banned from this network.\n\nReason: {reason}\nExpires: {expires} (in {duration})"
                        + "\nBy: {actor}\nCase: {id}",
                "You are banned from this server.\n\nReason: {reason}\nBy: {actor}\nCase: {id}",
                "You are banned from this server.\n\nReason: {reason}\nExpires: {expires} (in {duration})"
                        + "\nBy: {actor}\nCase: {id}",
                "You are muted. Reason: {reason}",
                "You are muted for another {duration}. Reason: {reason}",
                "You have been warned by {actor}. Reason: {reason}",
                "{player} was {kind} by {actor}: {reason}",
                "{player} was {kind}: {reason}");
    }

    /**
     * Дополнить пропуски встроенными текстами.
     *
     * <p>Gson оставляет {@code null} в полях, которых в ответе не было, — так
     * бывает после добавления шаблона на мастере, пока в базе лежит старый JSON.
     * Пустой экран бана — это кик молча, и допускать его нельзя.
     */
    public MessageTemplates complete() {
        MessageTemplates d = defaults();
        return new MessageTemplates(
                pick(banPermanent, d.banPermanent),
                pick(banTemporary, d.banTemporary),
                pick(serverBanPermanent, d.serverBanPermanent),
                pick(serverBanTemporary, d.serverBanTemporary),
                pick(mutePermanent, d.mutePermanent),
                pick(muteTemporary, d.muteTemporary),
                pick(warnNotice, d.warnNotice),
                // Молчаливое объявление — законный выбор, поэтому здесь пустая
                // строка сохраняется, а подставляется только отсутствие поля.
                broadcast == null ? d.broadcast : broadcast,
                pick(actorReceipt, d.actorReceipt));
    }

    /** Шаблон под наказание: вид плюс «навсегда или нет». */
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
