package dev.noro.agent.core;

import java.util.Arrays;

/**
 * Разбор команды наказания: {@code /ban Steve 7d 1.1 mass advertising}.
 *
 * <p>Срок и код правила необязательны и узнаются по форме, а не по позиции:
 * модератор в игре печатает быстро, и требовать от него плейсхолдеров вроде
 * {@code -} на месте пропущенного срока значит получать опечатки вместо банов.
 * Порядок между собой у них тоже свободный — {@code 7d 1.1} и {@code 1.1 7d}
 * означают одно и то же.
 *
 * <p>Код правила пишется как {@code 1.1} или {@code @chat.spam}: точка в слове
 * отличает его от первого слова причины. Без правила мастер откажет всем, у кого
 * нет {@code noro.mod.punish.bypass}, — так и задумано, наказание должно
 * ссылаться на свод.
 *
 * <p>Причина может быть пустой, если правило названо: текст тогда собирается из
 * шаблона мастера ({@code reason_by_rule}), а не придумывается здесь.
 */
public record PunishArgs(String target, Long minutes, String ruleCode, String reason) {

    /** Минимум, который примет мастер. Короче — это опечатка, а не причина. */
    private static final int MIN_REASON = 3;

    public static String usage(String command, boolean timed) {
        return "§cUsage: /" + command + " <player> " + (timed ? "[30m|7d|perm] " : "") + "[rule] <reason>";
    }

    /**
     * @param timed принимает ли команда срок ({@code /warn} — нет)
     * @return {@code null}, если разобрать нечего — тогда показывается подсказка
     */
    public static PunishArgs parse(String[] args, boolean timed) {
        if (args.length < 2) {
            return null;
        }
        String target = args[0];
        Long minutes = null;
        String ruleCode = null;
        int index = 1;

        // Два необязательных слова в любом порядке: срок и правило.
        for (int step = 0; step < 2 && index < args.length; step++) {
            if (timed && minutes == null) {
                Long parsed = DurationArg.parse(args[index]);
                if (parsed != null) {
                    // PERMANENT и «срок не назвали» — одно и то же для мастера.
                    minutes = parsed == DurationArg.PERMANENT ? null : parsed;
                    index++;
                    continue;
                }
            }
            if (ruleCode == null) {
                String rule = ruleCode(args[index]);
                if (rule != null) {
                    ruleCode = rule;
                    index++;
                    continue;
                }
            }
            break;
        }

        String reason = String.join(" ", Arrays.copyOfRange(args, index, args.length)).strip();
        if (reason.isEmpty()) {
            // Пустую причину достроит вызывающий — из шаблона по правилу.
            return ruleCode == null ? null : new PunishArgs(target, minutes, ruleCode, "");
        }
        return reason.length() < MIN_REASON ? null : new PunishArgs(target, minutes, ruleCode, reason);
    }

    /** {@code @chat.spam} либо {@code 1.1} — точка отличает код от слова. */
    private static String ruleCode(String arg) {
        if (arg == null || arg.length() < 2) {
            return null;
        }
        if (arg.startsWith("@")) {
            return arg.substring(1);
        }
        return arg.matches("[A-Za-z0-9_]+(\\.[A-Za-z0-9_]+)+") ? arg : null;
    }
}
