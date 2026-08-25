package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.Test;

/**
 * Меню разбора глазами игрока: во что оно превращается после разбора разметки.
 *
 * <p>Проверяется вся цепочка, а не шаблон: дважды выходило так, что строка в
 * lang-файле была правильной, а в чат уезжали решётки — потому что ломался
 * разбор. Тест на шаблон этого не ловит, тест на спаны ловит.
 */
class CaseMenuTest {

    private static final UUID ID = UUID.fromString("00000000-0000-0000-0000-000000000001");

    private static CaseSession session() {
        return new CaseSession(
                ID, ID, "GrieferSteve", ID, "AlexBuilder", "ломает дом",
                "minecraft:overworld", 1.0, 2.0, 3.0, null, false);
    }

    private static List<TextSpan> spans() {
        return TextMarkup.parse(CaseMenu.render(session(), "ru"));
    }

    /** Ни одной решётки в тексте: цвет — это цвет, а не видимые символы. */
    @Test
    void leavesNoColorCodesInText() {
        for (TextSpan span : spans()) {
            assertTrue(!span.text().contains("#"), "в тексте осталась разметка: " + span.text());
        }
    }

    /** Каждая кнопка обрамлена видимыми скобками, и обе они кликабельны. */
    @Test
    void drawsBracketsAroundEveryButton() {
        List<TextSpan> spans = spans();
        long open = spans.stream().filter(s -> s.linked() && s.text().equals("[")).count();
        long close = spans.stream().filter(s -> s.linked() && s.text().equals("]")).count();

        // Место, к цели, к автору, назад, чат, вещи, следить, заморозить,
        // закрыть, вернуть — десять кнопок, автор жалобы у дела есть.
        assertEquals(10, open);
        assertEquals(open, close);
    }

    /** Команда за кнопкой — та, что понимает агент: без номера дела в строке. */
    @Test
    void linksRunCaseCommands() {
        List<String> urls = spans().stream()
                .filter(TextSpan::linked)
                .map(TextSpan::url)
                .distinct()
                .toList();

        assertTrue(urls.contains("cmd:/case tp place"), urls.toString());
        assertTrue(urls.contains("cmd:/case inv"), urls.toString());
        assertTrue(urls.contains("cmd:/case release"), urls.toString());
    }
}
