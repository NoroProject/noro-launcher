package dev.noro.client.staff;

import dev.noro.client.ui.Theme;
import net.minecraft.network.chat.Component;

/**
 * Цвет по состоянию дела.
 *
 * <p>Живёт в staff, а не в теме ядра: «открыто» и «в работе» — понятия разбора,
 * и ядру про них знать нечего. Тема даёт краски, смысл им придаёт функция.
 */
public final class CaseStyle {

    private CaseStyle() {}

    /**
     * Вес жалоб словами: «3 жалобы от 2 человек».
     *
     * <p>Форму слова выбирает код, а не lang-файл: у Minecraft нет правил
     * склонения, и «3 жалоб» вылезало бы в каждой второй строке очереди.
     * Английский от этого не страдает — там все три ключа одинаковы.
     */
    public static Component reports(long reports, long people) {
        return Component.translatable(
                "noro.cases.queue.reports." + plural(reports), reports, people);
    }

    /** Русские формы: 1 жалоба, 2–4 жалобы, 5+ жалоб. */
    private static String plural(long n) {
        long tens = Math.abs(n) % 100;
        long ones = tens % 10;
        if (tens >= 11 && tens <= 14) {
            return "many";
        }
        if (ones == 1) {
            return "one";
        }
        return ones >= 2 && ones <= 4 ? "few" : "many";
    }

    public static int statusColor(String status) {
        if (status == null) {
            return Theme.TEXT_MUTED;
        }
        return switch (status) {
            // Открытое дело ждёт человека — это и есть главное действие экрана.
            case "open" -> Theme.CTA;
            case "in_review" -> Theme.SUCCESS;
            default -> Theme.TEXT_MUTED;
        };
    }
}
