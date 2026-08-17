package dev.noro.agent.core;

import java.util.Locale;

/**
 * Срок наказания так, как его набирают в чате: {@code 30m}, {@code 7d},
 * {@code perm}.
 *
 * <p>Те же единицы, что показывает админка в рамках правила
 * ({@code punish_limits::minutes} на мастере) — модератор читает «up to 7d» в
 * подсказке и набирает ровно это.
 */
public final class DurationArg {

    /** Вечное наказание: мастеру уходит {@code minutes = null}. */
    public static final long PERMANENT = -1;

    private DurationArg() {}

    /**
     * @return минуты, {@link #PERMANENT} для вечного, либо {@code null}, если
     *     это вообще не срок — тогда слово принадлежит причине
     */
    public static Long parse(String token) {
        if (token == null || token.isBlank()) {
            return null;
        }
        String value = token.strip().toLowerCase(Locale.ROOT);
        if (value.equals("perm") || value.equals("permanent") || value.equals("forever")) {
            return PERMANENT;
        }

        int split = 0;
        while (split < value.length() && Character.isDigit(value.charAt(split))) {
            split++;
        }
        if (split == 0 || split == value.length()) {
            return null;
        }
        long amount;
        try {
            amount = Long.parseLong(value.substring(0, split));
        } catch (NumberFormatException e) {
            return null;
        }
        Long perUnit = minutesPerUnit(value.substring(split));
        if (perUnit == null || amount <= 0) {
            return null;
        }
        return amount * perUnit;
    }

    private static Long minutesPerUnit(String unit) {
        switch (unit) {
            case "m":
                return 1L;
            case "h":
                return 60L;
            case "d":
                return 1440L;
            case "w":
                return 10_080L;
            // Месяц — 30 дней, год — 365: то же, что считает мастер в рамках
            // правила. Календарь тут никому не нужен, а расхождение с рамкой
            // на день превратило бы «до 1mo» в отказ.
            case "mo":
                return 43_200L;
            case "y":
                return 525_600L;
            default:
                return null;
        }
    }

    /**
     * Остаток словами: {@code 40s}, {@code 1h 35m}, {@code forever}.
     *
     * <p>Секунды показываются только на последней минуте — там они и нужны:
     * иначе на экране висит «ещё 1 минута», и игрок думает, что мут застрял.
     */
    public static String remaining(java.time.Duration left) {
        if (left == null) {
            return "forever";
        }
        long seconds = left.toSeconds();
        if (seconds < 60) {
            return Math.max(seconds, 1) + "s";
        }
        return format(left.toMinutes());
    }

    /** Минуты словами: {@code 10080} → {@code 7d}, {@code 95} → {@code 1h 35m}. */
    public static String format(long minutes) {
        if (minutes < 0) {
            return "forever";
        }
        long[] sizes = {525_600, 43_200, 1440, 60, 1};
        String[] units = {"y", "mo", "d", "h", "m"};
        StringBuilder out = new StringBuilder();
        long left = minutes;
        for (int i = 0; i < sizes.length && out.length() < 8; i++) {
            long count = left / sizes[i];
            if (count > 0) {
                out.append(out.length() > 0 ? " " : "").append(count).append(units[i]);
                left -= count * sizes[i];
            }
        }
        return out.length() == 0 ? "1m" : out.toString();
    }
}
