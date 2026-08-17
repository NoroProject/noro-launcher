package dev.noro.agent.core;

import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.Locale;

/**
 * Подстановка в шаблон наказания.
 *
 * <p>Плейсхолдеры: {@code {player}}, {@code {reason}}, {@code {duration}},
 * {@code {expires}}, {@code {actor}}, {@code {rule}}, {@code {rule_title}},
 * {@code {rule_link}}, {@code {id}}, {@code {kind}}. Неизвестное остаётся как
 * есть — это опечатка администратора, и увидеть её в игре полезнее, чем
 * получить пустое место.
 *
 * <p>Шаблон подставляется дословно: агент не дописывает в него ни правил, ни
 * ссылок. Что показать и в каком порядке — решает тот, кто правит тексты в
 * админке, иначе собрать строку так, как он задумал, невозможно.
 */
public final class MessageRender {

    /** UTC и без секунд: игроки читают это в разных часовых поясах. */
    private static final DateTimeFormatter EXPIRES =
            DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm 'UTC'", Locale.ROOT).withZone(ZoneId.of("UTC"));

    /** Нечего показать — прочерк: пустое место в строке читается как сбой. */
    private static final String NOTHING = "—";

    private MessageRender() {}

    /**
     * @param ruleTitle название пункта свода, если агент его знает
     * @param rulesUrl база ссылки на свод с мастера; пусто — ссылок не будет
     */
    public static String render(
            String template, PunishmentInfo punishment, String playerName, String ruleTitle, String rulesUrl) {
        if (template == null || punishment == null) {
            return "";
        }
        String rule = clean(punishment.ruleCode());
        return template
                .replace("{player}", text(playerName))
                .replace("{reason}", text(punishment.reason()))
                .replace("{duration}", DurationArg.remaining(punishment.left()))
                .replace("{expires}", punishment.permanent() ? NOTHING : EXPIRES.format(punishment.expiresAt()))
                .replace("{actor}", punishment.actorLabel() == null ? "console" : punishment.actorLabel())
                .replace("{rule}", rule == null ? NOTHING : rule)
                .replace("{rule_title}", text(ruleTitle))
                .replace("{rule_link}", link(rule, rulesUrl))
                .replace("{id}", shortId(punishment))
                .replace("{kind}", verb(punishment.kind()));
    }

    /** Короткая форма для мест, где свод не при чём. */
    public static String render(String template, PunishmentInfo punishment, String playerName) {
        return render(template, punishment, playerName, null, null);
    }

    /**
     * Кликабельный код правила. Без адреса свода — просто код: ссылку на
     * угаданный домен игрок откроет через год и попадёт в никуда.
     */
    private static String link(String rule, String rulesUrl) {
        if (rule == null) {
            return NOTHING;
        }
        if (rulesUrl == null || rulesUrl.isBlank()) {
            return rule;
        }
        return "[" + rule + "](" + rulesUrl + "#rule-" + rule + ")";
    }

    private static String clean(String ruleCode) {
        if (ruleCode == null || ruleCode.isBlank()) {
            return null;
        }
        String out = ruleCode.strip();
        return out.startsWith("@") ? out.substring(1) : out;
    }

    /**
     * Номер дела для игрока — первые восемь знаков UUID.
     *
     * <p>Полный UUID в апелляции никто не перепишет без ошибки, а найти по
     * префиксу в панели можно однозначно: восьми знаков хватает.
     */
    private static String shortId(PunishmentInfo punishment) {
        return punishment.id() == null ? NOTHING : punishment.id().toString().substring(0, 8);
    }

    /** Вид наказания словом, каким его читают в объявлении: «was banned». */
    private static String verb(String kind) {
        if (kind == null) {
            return "punished";
        }
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

    private static String text(String value) {
        return value == null ? "" : value;
    }
}
