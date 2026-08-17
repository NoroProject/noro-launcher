package dev.noro.agent.core;

import java.util.Arrays;

/**
 * Разбор команды наказания: {@code /ban Steve 7d @chat.spam mass advertising}.
 *
 * <p>Срок и код правила — необязательные и узнаются по форме, а не по позиции:
 * модератор в игре печатает быстро, и требовать от него плейсхолдеров вроде
 * {@code -} на месте пропущенного срока значит получать опечатки вместо банов.
 *
 * <p>Правило начинается с {@code @}: {@code @chat.spam}. Без него мастер
 * откажет всем, у кого нет {@code noro.mod.punish.bypass}, — так и задумано,
 * наказание должно ссылаться на свод.
 */
public record PunishArgs(String target, Long minutes, String ruleCode, String reason) {

    /** Что показать, когда разобрать не удалось. */
    public static String usage(String command, boolean timed) {
        return "§cUsage: /" + command + " <player> " + (timed ? "[30m|7d|perm] " : "") + "[@rule] <reason>";
    }

    /**
     * @param timed принимает ли команда срок ({@code /warn} — нет)
     * @return {@code null}, если аргументов не хватает
     */
    public static PunishArgs parse(String[] args, boolean timed) {
        if (args.length < 2) {
            return null;
        }
        String target = args[0];
        int index = 1;

        Long minutes = null;
        if (timed) {
            Long parsed = DurationArg.parse(args[index]);
            if (parsed != null) {
                // PERMANENT и «срок не назвали» — это одно и то же для мастера:
                // null минут значит навсегда.
                minutes = parsed == DurationArg.PERMANENT ? null : parsed;
                index++;
            }
        }

        String ruleCode = null;
        if (index < args.length && args[index].startsWith("@") && args[index].length() > 1) {
            ruleCode = args[index].substring(1);
            index++;
        }

        String reason = String.join(" ", Arrays.copyOfRange(args, index, args.length)).strip();
        if (reason.length() < 3) {
            return null;
        }
        return new PunishArgs(target, minutes, ruleCode, reason);
    }
}
