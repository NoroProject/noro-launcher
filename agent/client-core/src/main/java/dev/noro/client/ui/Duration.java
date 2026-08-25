package dev.noro.client.ui;

import java.util.Locale;

/**
 * Срок наказания строкой: {@code 15m}, {@code 2h}, {@code 7d}, {@code 1d 6h}.
 *
 * <p>Тот же формат, что принимает форма на сайте, и те же правила: пусто —
 * навсегда, непонятная строка — не ноль, а отказ. Ноль здесь означал бы
 * «наказать на нисколько», и разница видна только когда уже поздно.
 */
public final class Duration {

    private Duration() {}

    /**
     * Минуты или {@code null} — «навсегда». Возвращает {@code -1}, если строку
     * разобрать не удалось: вызывающий гасит кнопку, а не отправляет наугад.
     */
    public static long parse(String text) {
        String value = text == null ? "" : text.trim().toLowerCase(Locale.ROOT);
        if (value.isEmpty()) {
            return Long.MIN_VALUE; // навсегда
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
        // Число без единицы — минуты: «30» в поле срока значит полчаса.
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
