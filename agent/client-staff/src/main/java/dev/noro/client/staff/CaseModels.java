package dev.noro.client.staff;

import com.google.gson.JsonObject;
import java.util.List;
import java.util.Map;

/**
 * Формы карточки дела — один в один с {@code crates/mod_link}.
 *
 * <p>Это договор с лаунчером, а не с мастером: мод не знает ни URL админки, ни
 * порядка её вызовов. Поля, которых у старого лаунчера нет, приезжают {@code
 * null} — Gson не ругается, и панель показывает, что есть.
 */
public final class CaseModels {

    private CaseModels() {}

    /** Строка очереди: столько, сколько нужно, чтобы выбрать следующее дело. */
    public record Brief(
            String id,
            long number,
            String target_id,
            String target_name,
            String game_server_id,
            String server_name,
            String status,
            String claimed_by,
            String claimed_by_name,
            String opened_at,
            String resolved_at,
            String verdict,
            String rule_code,
            long reports_count,
            long reporters_count,
            String last_report_at) {

        /** Человеческий номер дела: {@code N-000000001}. */
        public String label() {
            return String.format("N-%09d", number);
        }
    }

    /**
     * Событие ленты. {@code payload} остаётся сырым: у двух десятков видов
     * событий нет общей формы, и плоская запись со всеми полями всех событий —
     * ровно та ошибка, которой стоило избежать.
     */
    public record Event(
            String id,
            String at,
            String actor_label,
            String source,
            String kind,
            JsonObject payload) {}

    public record Message(String id, String at, String sender_name, String channel, String content) {}

    public record Report(
            String id,
            String reporter_id,
            String reporter_name,
            String reason,
            String world,
            Double x,
            Double y,
            Double z,
            String created_at) {}

    public record Punishment(
            String id,
            String kind,
            String reason,
            String created_at,
            String expires_at,
            String revoked_at,
            String rule_code) {}

    /** Сколько жалоб человека подтвердилось — вес его слова в очереди. */
    public record ReporterStats(long total, long confirmed, long rejected) {}

    /** Карточка целиком: лента, жалобы, наказания и срез чата приходят вместе. */
    public record View(
            @com.google.gson.annotations.SerializedName("case") Brief brief,
            List<Report> reports,
            List<Event> events,
            List<Punishment> punishments,
            List<Message> messages,
            boolean chat_allowed,
            Map<String, ReporterStats> reporters) {}

    /**
     * Занятый слот снимка инвентаря.
     *
     * <p>{@code nbt} — предмет целиком в JSON, как его отдал сервер: по нему
     * рисуется настоящая иконка с зачарованиями и переименованием. Пусто —
     * платформа так не умеет, и остаётся имя строкой.
     */
    public record Slot(int slot, String id, int count, String name, String nbt) {}

    /** Досье игрока под прицелом. */
    public record Dossier(
            String user_id,
            String username,
            List<String> roles,
            String first_seen,
            long cases_total,
            long cases_confirmed,
            List<Punishment> active_punishments) {}
}
