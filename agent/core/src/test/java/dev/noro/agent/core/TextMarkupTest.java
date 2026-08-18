package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

/**
 * Разбор разметки шаблонов. Он один на Paper и на моды, поэтому проверяется
 * здесь: разъедься он по платформам — и один текст выглядел бы в игре по-разному.
 */
class TextMarkupTest {

    @Test
    void readsWebHexColor() {
        List<TextSpan> spans = TextMarkup.parse("#f87171Забанен");

        assertEquals(1, spans.size());
        assertEquals("Забанен", spans.get(0).text());
        assertEquals(0xf87171, spans.get(0).color());
    }

    @Test
    void readsLegacyCodesAndStyles() {
        List<TextSpan> spans = TextMarkup.parse("&cкрасный&lжирный");

        assertEquals(2, spans.size());
        assertEquals(0xFF5555, spans.get(0).color());
        assertFalse(spans.get(0).bold());
        assertTrue(spans.get(1).bold(), "стиль после цвета продолжает тот же цвет");
        assertEquals(0xFF5555, spans.get(1).color());
    }

    /** Цвет сбрасывает стили — так же, как ванильный разбор legacy-строк. */
    @Test
    void colorResetsStyles() {
        List<TextSpan> spans = TextMarkup.parse("&lжирный&cобычный");

        assertTrue(spans.get(0).bold());
        assertFalse(spans.get(1).bold());
    }

    /** Так цвет роли приходит из префиксов и от LuckPerms. */
    @Test
    void readsLegacyHexSequence() {
        List<TextSpan> spans = TextMarkup.parse("§x§f§f§8§c§8§2★");

        assertEquals(1, spans.size());
        assertEquals(0xff8c82, spans.get(0).color());
        assertEquals("★", spans.get(0).text());
    }

    @Test
    void makesMarkdownAndRawLinksClickable() {
        List<TextSpan> markdown = TextMarkup.parse("Правило [1.1](https://noro.example/rules#rule-1.1)");
        assertEquals(2, markdown.size());
        assertEquals("1.1", markdown.get(1).text());
        assertEquals("https://noro.example/rules#rule-1.1", markdown.get(1).url());

        List<TextSpan> raw = TextMarkup.parse("Апелляция: https://noro.example/support");
        assertTrue(raw.get(raw.size() - 1).linked());
        assertEquals("https://noro.example/support", raw.get(raw.size() - 1).url());
    }

    /** Одинокий амперсанд остаётся собой, а не съедается разбором. */
    @Test
    void keepsPlainTextIntact() {
        assertEquals("Tom & Jerry", TextMarkup.parse("Tom & Jerry").get(0).text());
    }

    /**
     * За цветом сразу идёт текст, и он может начинаться с a–f: «#f8fafcdalynkaa»
     * — это цвет и ник. Ровно на этом ник слипался с кодом цвета в чате.
     */
    @Test
    void colorEndsAfterSixDigits() {
        List<TextSpan> spans = TextMarkup.parse("#f8fafcdalynkaa");

        assertEquals(1, spans.size());
        assertEquals(0xf8fafc, spans.get(0).color());
        assertEquals("dalynkaa", spans.get(0).text());
    }

    @Test
    void emptyMarkupGivesNoSpans() {
        assertTrue(TextMarkup.parse("").isEmpty());
        assertTrue(TextMarkup.parse(null).isEmpty());
    }
}
