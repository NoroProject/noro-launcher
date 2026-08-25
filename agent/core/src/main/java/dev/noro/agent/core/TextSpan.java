package dev.noro.agent.core;

/**
 * Кусок текста с одним стилем — результат разбора разметки шаблона.
 *
 * <p>Платформа превращает такие куски в свой компонент: мод — в ванильный
 * {@code Component}, Paper — в Adventure. Разбор при этом один, в
 * {@link TextMarkup}: раньше цвета понимал только мод, и те же шаблоны на Paper
 * выводились строкой «#f87171ДОСТУП…».
 *
 * @param color {@code 0xrrggbb} либо {@code -1}, если цвет не задан
 * @param url ссылка, если этот кусок кликабельный
 * @param font шрифт вида {@code noro:prefix} либо {@code null}. Им плашка роли
 *        отличается от обычного текста: тот же символ в другом шрифте — другая
 *        картинка
 */
public record TextSpan(
        String text,
        int color,
        boolean bold,
        boolean italic,
        boolean underlined,
        boolean strikethrough,
        boolean obfuscated,
        String url,
        String font) {

    /** Цвет не задан — кусок наследует цвет контекста. */
    public static final int NO_COLOR = -1;

    public boolean hasColor() {
        return color != NO_COLOR;
    }

    public boolean linked() {
        return url != null && !url.isEmpty();
    }

    public boolean hasFont() {
        return font != null && !font.isEmpty();
    }

    /** Тот же кусок, но кликабельный: так метка ссылки собирается из разметки. */
    TextSpan linkedTo(String url) {
        return linkedTo(url, color);
    }

    TextSpan linkedTo(String url, int color) {
        return new TextSpan(text, color, bold, italic, underlined, strikethrough, obfuscated, url, font);
    }
}
