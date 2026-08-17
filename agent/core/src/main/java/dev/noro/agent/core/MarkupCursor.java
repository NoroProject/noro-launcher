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
                pending.toString(), color, bold, italic, underlined, strikethrough, obfuscated, url));
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
     */
    void linked(String label, String url) {
        flush();
        pending.append(label);
        flush(url);
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
