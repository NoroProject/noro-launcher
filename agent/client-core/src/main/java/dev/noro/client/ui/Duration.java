package dev.noro.client.ui;

import java.util.Locale;

/**
 * Punishment durations as text: {@code 15m}, {@code 2h}, {@code 7d}, {@code 1d 6h}.
 * Same format the site's form accepts.
 */
public final class Duration {

    private Duration() {}

    /**
     * Minutes; {@code Long.MIN_VALUE} for forever, {@code -1} when the string
     * didn't parse. An unparseable string must not fall through to zero — that
     * would read as "punish for no time at all" and the difference only shows up
     * once it's too late.
     */
    public static long parse(String text) {
        String value = text == null ? "" : text.trim().toLowerCase(Locale.ROOT);
        if (value.isEmpty()) {
            return Long.MIN_VALUE; // forever
        }
        long total = 0;
        long number = -1;
        for (int i = 0; i < value.length(); i++) {
            char c = value.charAt(i);
            if (Character.isDigit(c)) {
                number = (number < 0 ? 0 : number) * 10 + (c - '0');
                continue;
            }
            if (c == ' ') {
                continue;
            }
            if (number < 0) {
                return -1;
            }
            long factor = switch (c) {
                case 'm' -> 1;
                case 'h' -> 60;
                case 'd' -> 60 * 24;
                case 'w' -> 60 * 24 * 7;
                default -> -1;
            };
            if (factor < 0) {
                return -1;
            }
            total += number * factor;
            number = -1;
        }
        // A bare number means minutes: "30" in the duration field is half an hour.
        if (number >= 0) {
            total += number;
        }
        return total == 0 ? -1 : total;
    }

    public static boolean forever(long minutes) {
        return minutes == Long.MIN_VALUE;
    }

    public static boolean broken(long minutes) {
        return minutes == -1;
    }
}
