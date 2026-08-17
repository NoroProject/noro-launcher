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
 */
public record TextSpan(
        String text,
        int color,
        boolean bold,
        boolean italic,
        boolean underlined,
        boolean strikethrough,
        boolean obfuscated,
        String url) {

    /** Цвет не задан — кусок наследует цвет контекста. */
    public static final int NO_COLOR = -1;

    public boolean hasColor() {
        return color != NO_COLOR;
    }

    public boolean linked() {
        return url != null && !url.isEmpty();
    }
}
