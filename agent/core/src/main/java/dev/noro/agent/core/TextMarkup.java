package dev.noro.agent.core;

import java.util.ArrayList;
import java.util.List;

/**
 * Разбор разметки шаблонов мастера в куски со стилем.
 *
 * <p>Понимает то, что администратор набирает в админке:
 * <ul>
 *   <li>{@code #f87171} и {@code &#f87171} — цвет из веб-палитры;</li>
 *   <li>{@code &c}, {@code &l}, {@code §c} — legacy-коды цвета и стиля;</li>
 *   <li>{@code §x§f§8§7§1§7§1} — legacy-hex: в таком виде цвет приходит из
 *       префиксов ролей и от LuckPerms;</li>
 *   <li>{@code [текст](ссылка)} и сырые {@code https://…} — кликабельные.</li>
 * </ul>
 *
 * <p>Живёт в core по той же причине, что и плейсхолдеры: разъедься разбор — и
 * один и тот же шаблон выглядел бы на Paper и на модах по-разному. Так и было,
 * пока цвета понимал только модовый парсер: на Paper в чат уезжала строка
 * «#f87171ДОСТУП…».
 */
public final class TextMarkup {

    private TextMarkup() {}

    public static List<TextSpan> parse(String text) {
        List<TextSpan> spans = new ArrayList<>();
        if (text == null || text.isEmpty()) {
            return spans;
        }
        MarkupCursor cursor = new MarkupCursor(spans);
        int at = 0;
        while (at < text.length()) {
            int consumed = read(text, at, cursor);
            if (consumed > 0) {
                at += consumed;
                continue;
            }
            cursor.append(text.charAt(at));
            at++;
        }
        cursor.flush();
        return spans;
    }

    /** @return сколько символов съел код, либо 0 — тогда это обычный текст */
    private static int read(String text, int at, MarkupCursor cursor) {
        char current = text.charAt(at);
        return switch (current) {
            case '§' -> legacy(text, at, cursor, true);
            case '&' -> text.startsWith("&#", at) ? webColor(text, at + 1, cursor, 1) : legacy(text, at, cursor, false);
            case '#' -> webColor(text, at, cursor, 0);
            case '[' -> link(text, at, cursor);
            case 'h', 'H' -> rawUrl(text, at, cursor);
            default -> 0;
        };
    }

    /** `§c`, `&l`, а также `§x§r§r§g§g§b§b` — шесть пар после `x`. */
    private static int legacy(String text, int at, MarkupCursor cursor, boolean section) {
        if (at + 1 >= text.length()) {
            return 0;
        }
        char code = Character.toLowerCase(text.charAt(at + 1));
        if (section && code == 'x') {
            int rgb = legacyHex(text, at);
            if (rgb != TextSpan.NO_COLOR) {
                cursor.color(rgb);
                return 14;
            }
        }
        if (code == 'r') {
            cursor.reset();
            return 2;
        }
        int color = MarkupCursor.legacyColor(code);
        if (color != TextSpan.NO_COLOR) {
            cursor.color(color);
            return 2;
        }
        if (MarkupCursor.styleCode(code)) {
            cursor.style(code);
            return 2;
        }
        // Не код, а сам символ: «Tom & Jerry» должен остаться собой.
        return 0;
    }

    private static int legacyHex(String text, int at) {
        if (at + 13 >= text.length()) {
            return TextSpan.NO_COLOR;
        }
        StringBuilder hex = new StringBuilder(6);
        for (int step = 0; step < 6; step++) {
            if (text.charAt(at + 2 + step * 2) != '§') {
                return TextSpan.NO_COLOR;
            }
            hex.append(text.charAt(at + 3 + step * 2));
        }
        return rgb(hex.toString());
    }

    /**
     * `#rrggbb`, необязательно с амперсандом впереди.
     *
     * <p>Цвет — ровно шесть знаков, и седьмой hex-цифрой быть не должен. Иначе
     * номер дела в шаблоне («Дело: #{id}» → `#1a2b3c4d`) съедался бы как цвет, и
     * игрок видел бы в бане обрубок вместо номера, по которому подаёт апелляцию.
     */
    private static int webColor(String text, int at, MarkupCursor cursor, int extra) {
        if (at + 7 > text.length()) {
            return 0;
        }
        if (at + 7 < text.length() && hexDigit(text.charAt(at + 7))) {
            return 0;
        }
        int rgb = rgb(text.substring(at + 1, at + 7));
        if (rgb == TextSpan.NO_COLOR) {
            return 0;
        }
        cursor.color(rgb);
        return 7 + extra;
    }

    private static boolean hexDigit(char ch) {
        return Character.digit(ch, 16) >= 0;
    }

    /** `[текст](ссылка)` — так в шаблон кладут ссылку на свод правил. */
    private static int link(String text, int at, MarkupCursor cursor) {
        int label = text.indexOf(']', at);
        if (label < 0 || label + 1 >= text.length() || text.charAt(label + 1) != '(') {
            return 0;
        }
        int end = text.indexOf(')', label + 2);
        if (end < 0) {
            return 0;
        }
        cursor.linked(text.substring(at + 1, label), text.substring(label + 2, end));
        return end - at + 1;
    }

    private static int rawUrl(String text, int at, MarkupCursor cursor) {
        if (!text.regionMatches(true, at, "http://", 0, 7) && !text.regionMatches(true, at, "https://", 0, 8)) {
            return 0;
        }
        int end = at;
        while (end < text.length() && !Character.isWhitespace(text.charAt(end)) && text.charAt(end) != ')') {
            end++;
        }
        String url = text.substring(at, end);
        cursor.linked(url, url);
        return end - at;
    }

    private static int rgb(String hex) {
        try {
            return hex.length() == 6 ? Integer.parseInt(hex, 16) : TextSpan.NO_COLOR;
        } catch (NumberFormatException e) {
            return TextSpan.NO_COLOR;
        }
    }
}
