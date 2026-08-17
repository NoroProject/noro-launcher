package dev.noro.agent.core;

import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.Locale;

/**
 * Подстановка в шаблон наказания.
 *
 * <p>Плейсхолдеры: {@code {player}}, {@code {reason}}, {@code {duration}},
 * {@code {expires}}, {@code {actor}}, {@code {rule}}, {@code {id}},
 * {@code {kind}}. Неизвестное в шаблоне остаётся как есть — это опечатка
 * администратора, и увидеть её в игре полезнее, чем получить пустое место.
 */
public final class MessageRender {

    /** UTC и без секунд: игроки читают это в разных часовых поясах. */
    private static final DateTimeFormatter EXPIRES =
            DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm 'UTC'", Locale.ROOT).withZone(ZoneId.of("UTC"));

    private MessageRender() {}

    public static String render(String template, PunishmentInfo punishment, String playerName) {
        if (template == null) {
            return "";
        }
        String out = template
                .replace("{player}", playerName == null ? "" : playerName)
                .replace("{reason}", punishment.reason() == null ? "" : punishment.reason())
                .replace("{duration}", DurationArg.format(punishment.minutesLeft()))
                .replace("{expires}", punishment.permanent() ? "never" : EXPIRES.format(punishment.expiresAt()))
                .replace("{actor}", punishment.actorLabel() == null ? "console" : punishment.actorLabel())
                .replace("{rule}", punishment.ruleCode() == null ? "—" : punishment.ruleCode())
                .replace("{id}", shortId(punishment))
                .replace("{kind}", verb(punishment.kind()));
        return PrefixFormat.legacy(out);
    }

    /**
     * Номер дела для игрока — первые восемь знаков UUID.
     *
     * <p>Полный UUID в апелляции никто не перепишет без ошибки, а найти по
     * префиксу в панели можно однозначно: восьми знаков хватает.
     */
    private static String shortId(PunishmentInfo punishment) {
        return punishment.id() == null ? "—" : punishment.id().toString().substring(0, 8);
    }

    /** Вид наказания словом, каким его читают в объявлении: «was banned». */
    private static String verb(String kind) {
        switch (kind) {
            case "ban":
            case "server_ban":
                return "banned";
            case "mute":
                return "muted";
            default:
                return "warned";
        }
    }

}
