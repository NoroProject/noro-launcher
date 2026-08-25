package dev.noro.agent.core;

import java.util.Locale;

/**
 * Один тег MiniMessage: {@code <red>}, {@code <#ff8c82>}, {@code <bold>},
 * {@code <font:noro:prefix>}, {@code <click:run_command:'/case chat'>}.
 *
 * <p>Свой разбор, а не Adventure: её MiniMessage живёт в Paper, а на модах
 * Adventure нет вовсе. Один формат на обеих платформах важнее, чем чужая
 * реализация на одной из них.
 *
 * <p>Понимается подмножество, которого хватает шаблонам: цвета, начертания,
 * шрифт, клик и сброс. Незнакомый тег остаётся текстом — так опечатка в шаблоне
 * видна в игре, а не съедается молча.
 */
final class MiniTag {

    /** Именованные цвета MiniMessage — те же шестнадцать, что и в legacy. */
    private static final String[] NAMES = {
        "black", "dark_blue", "dark_green", "dark_aqua", "dark_red", "dark_purple", "gold", "gray",
        "dark_gray", "blue", "green", "aqua", "red", "light_purple", "yellow", "white",
    };

    private MiniTag() {}

    /**
     * Применить тег к курсору.
     *
     * @param body содержимое угловых скобок, без них самих
     * @return {@code false} — тег незнакомый, пусть остаётся текстом
     */
    static boolean apply(String body, MarkupCursor cursor) {
        if (body.isEmpty()) {
            return false;
        }
        if (body.charAt(0) == '/') {
            cursor.pop();
            return true;
        }
        String name = body.toLowerCase(Locale.ROOT);
        int colon = name.indexOf(':');
        String head = colon < 0 ? name : name.substring(0, colon);
        String rest = colon < 0 ? "" : body.substring(colon + 1);

        return switch (head) {
            case "reset" -> {
                cursor.reset();
                yield true;
            }
            case "font" -> {
                cursor.push();
                cursor.font(rest);
                yield true;
            }
            case "click" -> click(rest, cursor);
            default -> style(name, cursor);
        };
    }

    /** Цвет либо начертание — то, что задаётся одним словом. */
    private static boolean style(String name, MarkupCursor cursor) {
        int rgb = color(name);
        if (rgb != TextSpan.NO_COLOR) {
            cursor.push();
            cursor.tint(rgb);
            return true;
        }
        char code = decoration(name);
        if (code != 0) {
            cursor.push();
            cursor.decorate(code);
            return true;
        }
        return false;
    }

    /** {@code <#ff8c82>} и {@code <red>}. */
    private static int color(String name) {
        if (name.startsWith("#") && name.length() == 7) {
            try {
                return Integer.parseInt(name.substring(1), 16);
            } catch (NumberFormatException e) {
                return TextSpan.NO_COLOR;
            }
        }
        for (int i = 0; i < NAMES.length; i++) {
            if (NAMES[i].equals(name)) {
                return MarkupCursor.legacyColor("0123456789abcdef".charAt(i));
            }
        }
        return TextSpan.NO_COLOR;
    }

    private static char decoration(String name) {
        return switch (name) {
            case "bold", "b" -> 'l';
            case "italic", "i", "em" -> 'o';
            case "underlined", "u" -> 'n';
            case "strikethrough", "st" -> 'm';
            case "obfuscated", "obf" -> 'k';
            default -> 0;
        };
    }

    /**
     * {@code <click:run_command:'/case chat'>} и {@code <click:open_url:'…'>}.
     *
     * <p>Команда уезжает с приставкой {@code cmd:} — в этом виде её и ждёт
     * платформа, различая клик-команду и клик-ссылку.
     */
    private static boolean click(String rest, MarkupCursor cursor) {
        int colon = rest.indexOf(':');
        if (colon < 0) {
            return false;
        }
        String kind = rest.substring(0, colon).toLowerCase(Locale.ROOT);
        String value = unquote(rest.substring(colon + 1));
        cursor.push();
        switch (kind) {
            case "run_command", "suggest_command" -> cursor.link("cmd:" + value);
            case "open_url" -> cursor.link(value);
            default -> {
                cursor.pop();
                return false;
            }
        }
        return true;
    }

    private static String unquote(String value) {
        String trimmed = value.trim();
        if (trimmed.length() >= 2
                && (trimmed.charAt(0) == '\'' || trimmed.charAt(0) == '"')
                && trimmed.charAt(trimmed.length() - 1) == trimmed.charAt(0)) {
            return trimmed.substring(1, trimmed.length() - 1);
        }
        return trimmed;
    }
}
