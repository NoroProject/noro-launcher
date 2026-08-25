package dev.noro.agent.core;

import java.util.List;

/**
 * Состояние разбора разметки: накопленный текст и текущий стиль.
 *
 * <p>Отдельно от {@link TextMarkup}, потому что там читаются коды, а здесь
 * копится результат: вместе это был бы файл, в котором не найти ни того, ни
 * другого.
 */
final class MarkupCursor {

    /** Цвета legacy-кодов, по порядку `0`–`f`. */
    private static final int[] LEGACY_COLORS = {
        0x000000, 0x0000AA, 0x00AA00, 0x00AAAA, 0xAA0000, 0xAA00AA, 0xFFAA00, 0xAAAAAA,
        0x555555, 0x5555FF, 0x55FF55, 0x55FFFF, 0xFF5555, 0xFF55FF, 0xFFFF55, 0xFFFFFF,
    };

    private final List<TextSpan> spans;
    private final StringBuilder pending = new StringBuilder();

    private int color = TextSpan.NO_COLOR;
    private boolean bold;
    private boolean italic;
    private boolean underlined;
    private boolean strikethrough;
    private boolean obfuscated;
    private String font;
    private String link;

    /**
     * Стопка стилей для парных тегов MiniMessage.
     *
     * <p>Legacy-коды действуют «до отмены», а теги — до своего закрытия:
     * {@code <red>алый <bold>жирный</bold> снова алый</red>}. Без стопки
     * закрывающий тег пришлось бы понимать как полный сброс, и вложенность
     * сломалась бы на первом же случае.
     */
    private final java.util.ArrayDeque<State> stack = new java.util.ArrayDeque<>();

    private record State(
            int color,
            boolean bold,
            boolean italic,
            boolean underlined,
            boolean strikethrough,
            boolean obfuscated,
            String font,
            String link) {}

    MarkupCursor(List<TextSpan> spans) {
        this.spans = spans;
    }

    void append(char ch) {
        pending.append(ch);
    }

    /** Закрыть текущий кусок. Пустой не добавляем: пустых спанов не бывает. */
    void flush() {
        flush(null);
    }

    private void flush(String url) {
        if (pending.length() == 0) {
            return;
        }
        spans.add(new TextSpan(
                pending.toString(),
                color,
                bold,
                italic,
                underlined,
                strikethrough,
                obfuscated,
                url == null ? link : url,
                font));
        pending.setLength(0);
    }

    /**
     * Цвет сбрасывает накопленные стили: так же поступает ванильный разбор
     * legacy-строк, и расходиться с ним нельзя — иначе `&c&lтекст` и
     * `&l&cтекст` дали бы разный результат в игре и у нас.
     */
    void color(int rgb) {
        flush();
        color = rgb;
        clearStyles();
    }

    void reset() {
        flush();
        color = TextSpan.NO_COLOR;
        clearStyles();
    }

    void style(char code) {
        flush();
        switch (code) {
            case 'l' -> bold = true;
            case 'o' -> italic = true;
            case 'n' -> underlined = true;
            case 'm' -> strikethrough = true;
            default -> obfuscated = true;
        }
    }

    /**
     * Кликабельный кусок. Ссылка не переходит на следующий текст: подчёркнутым и
     * кликабельным должен быть ровно тот фрагмент, который её описывает.
     *
     * <p>Метка приходит уже разобранной: внутри неё работает та же разметка, что
     * снаружи. Пока не работала, кнопки меню разбора уезжали в чат строкой
     * «#e6e6e6чат#8b8b8b» — решётки было видно, цвета нет.
     *
     * <p>Кусок без своего цвета берёт цвет вокруг ссылки: «[чат]» в серой строке
     * должен остаться серым, а не стать белым только потому, что он кликабельный.
     */
    /** Голая ссылка: метки нет, разбирать нечего — текст и есть адрес. */
    void linked(String url) {
        flush();
        pending.append(url);
        flush(url);
    }

    void linked(List<TextSpan> label, String url) {
        flush();
        for (TextSpan span : label) {
            spans.add(span.hasColor() ? span.linkedTo(url) : span.linkedTo(url, color));
        }
    }

    /** Запомнить текущий стиль: открылся парный тег. */
    void push() {
        flush();
        stack.push(new State(color, bold, italic, underlined, strikethrough, obfuscated, font, link));
    }

    /** Вернуть стиль, каким он был до тега. Лишний закрывающий тег — не беда. */
    void pop() {
        flush();
        State was = stack.poll();
        if (was == null) {
            return;
        }
        color = was.color();
        bold = was.bold();
        italic = was.italic();
        underlined = was.underlined();
        strikethrough = was.strikethrough();
        obfuscated = was.obfuscated();
        font = was.font();
        link = was.link();
    }

    /** Цвет без сброса стилей: у тегов вложенность своя, стопка её и держит. */
    void tint(int rgb) {
        flush();
        color = rgb;
    }

    void decorate(char code) {
        flush();
        switch (code) {
            case 'l' -> bold = true;
            case 'o' -> italic = true;
            case 'n' -> underlined = true;
            case 'm' -> strikethrough = true;
            default -> obfuscated = true;
        }
    }

    void font(String value) {
        flush();
        font = value;
    }

    /** Кликабельность на весь кусок до закрывающего тега. */
    void link(String value) {
        flush();
        link = value;
    }

    private void clearStyles() {
        bold = false;
        italic = false;
        underlined = false;
        strikethrough = false;
        obfuscated = false;
    }

    static int legacyColor(char code) {
        int index = "0123456789abcdef".indexOf(Character.toLowerCase(code));
        return index < 0 ? TextSpan.NO_COLOR : LEGACY_COLORS[index];
    }

    static boolean styleCode(char code) {
        return "klmno".indexOf(Character.toLowerCase(code)) >= 0;
    }
}
