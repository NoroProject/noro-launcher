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

    /**
     * Внутри метки ссылки работает та же разметка, что снаружи.
     *
     * <p>Пока не работала, кнопки меню разбора уезжали в чат строкой
     * «#e6e6e6чат#8b8b8b»: цвет был виден решётками, а сама кнопка — серой.
     */
    @Test
    void parsesMarkupInsideLinkLabel() {
        List<TextSpan> spans = TextMarkup.parse("#8b8b8b [#e6e6e6чат#8b8b8b](cmd:/case chat)");

        List<TextSpan> linked = spans.stream().filter(TextSpan::linked).toList();
        assertEquals(1, linked.size());
        assertEquals("чат", linked.get(0).text());
        assertEquals(0xe6e6e6, linked.get(0).color());
        assertEquals("cmd:/case chat", linked.get(0).url());
    }

    /**
     * Кнопка меню разбора: скобки видны в чате и кликаются вместе со словом.
     *
     * <p>Метка кончается на `](`, поэтому внутренняя `]` остаётся меткой, а не
     * обрывает ссылку на середине.
     */
    @Test
    void keepsBracketsInsideButtonLabel() {
        List<TextSpan> spans = TextMarkup.parse("[#b8c4e0[#f3e7b3чат#b8c4e0]](cmd:/case chat)");

        assertEquals(3, spans.size());
        assertEquals("[", spans.get(0).text());
        assertEquals("чат", spans.get(1).text());
        assertEquals("]", spans.get(2).text());
        assertEquals(0xf3e7b3, spans.get(1).color());
        // Кликается вся кнопка: попасть мышью в три буквы тяжелее, чем кажется.
        assertTrue(spans.stream().allMatch(TextSpan::linked));
    }

    /** Метка без своего цвета берёт цвет вокруг ссылки, а не сбрасывает его. */
    @Test
    void linkLabelInheritsSurroundingColor() {
        List<TextSpan> spans = TextMarkup.parse("#8b8b8b[чат](cmd:/case chat)");

        TextSpan link = spans.get(spans.size() - 1);
        assertTrue(link.linked());
        assertEquals(0x8b8b8b, link.color());
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

    /** Именованный цвет и hex MiniMessage — то же самое, что legacy-коды. */
    @Test
    void readsMiniMessageColors() {
        assertEquals(0xFF5555, TextMarkup.parse("<red>стоп").get(0).color());
        assertEquals(0xff8c82, TextMarkup.parse("<#ff8c82>роль").get(0).color());
        assertEquals("стоп", TextMarkup.parse("<red>стоп").get(0).text());
    }

    /**
     * Парный тег действует до своего закрытия, а не до конца строки.
     *
     * <p>В этом и разница с legacy: `&l` включает жирный навсегда, `<bold>` —
     * только внутри себя.
     */
    @Test
    void closesPairedTags() {
        List<TextSpan> spans = TextMarkup.parse("<red>алый <bold>жирный</bold> снова алый");

        assertEquals(3, spans.size());
        assertTrue(spans.get(1).bold(), "внутри тега жирный");
        assertTrue(!spans.get(2).bold(), "после закрытия жирность снята");
        assertEquals(0xFF5555, spans.get(2).color(), "цвет пережил вложенный тег");
    }

    /** Шрифт: им плашка роли отличается от обычного текста. */
    @Test
    void readsFontTag() {
        List<TextSpan> spans = TextMarkup.parse("<font:noro:prefix>\uE000</font> Steve");

        assertEquals("noro:prefix", spans.get(0).font());
        assertTrue(!spans.get(1).hasFont(), "после закрытия шрифт обычный");
    }

    /** Клик тегом — то же, что и ссылкой в квадратных скобках. */
    @Test
    void readsClickTag() {
        List<TextSpan> spans = TextMarkup.parse("<click:run_command:'/case chat'>чат</click>");

        assertTrue(spans.get(0).linked());
        assertEquals("cmd:/case chat", spans.get(0).url());
    }

    /** «1 < 2» остаётся собой: незнакомый тег не съедается. */
    @Test
    void leavesPlainAngleBracketsAlone() {
        assertEquals("1 < 2", TextMarkup.parse("1 < 2").get(0).text());
        assertEquals("<неведомый>", TextMarkup.parse("<неведомый>").get(0).text());
    }
}
