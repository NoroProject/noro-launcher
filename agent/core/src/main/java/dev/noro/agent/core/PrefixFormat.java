package dev.noro.agent.core;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Сборка префикса роли в legacy-формате Minecraft.
 *
 * <p>{@code #ff8c82} + {@code ★} превращается в {@code §x§f§f§8§c§8§2★§r}.
 * Формат {@code §x} с шестью «§-цифрами» — единственный способ передать
 * произвольный RGB там, где ждут legacy-коды; LuckPerms хранит префикс строкой
 * и разбирает его именно так.
 */
public final class PrefixFormat {

    private static final Pattern HEX = Pattern.compile("#[0-9a-fA-F]{6}");
    private static final Pattern HEX_COLOR = Pattern.compile("&#?([0-9a-fA-F]{6})|#([0-9a-fA-F]{6})");
    private static final char SECTION = '§';
    private static final String RESET = SECTION + "r";
    private static final String CODES = "0123456789abcdefklmnorABCDEFKLMNOR";

    private PrefixFormat() {}

    /**
     * @return префикс либо {@code null}, если у роли нет иконки — такая роль
     *         в игре не показывается
     */
    public static String of(String colorHex, String icon) {
        if (icon == null || icon.isBlank()) {
            return null;
        }
        String color = color(colorHex);
        return color + icon + RESET;
    }

    /**
     * {@code &c} → {@code §c}, {@code #f87171} → {@code §x§f§8§7§1§7§1}.
     * В админке цвета набирают амперсандом и hex-кодами.
     */
    public static String legacy(String text) {
        if (text == null) {
            return "";
        }
        Matcher matcher = HEX_COLOR.matcher(text);
        StringBuilder hexConverted = new StringBuilder(text.length());
        while (matcher.find()) {
            String hexStr = matcher.group(1) != null ? matcher.group(1) : matcher.group(2);
            StringBuilder replacement = new StringBuilder(14).append(SECTION).append('x');
            for (int i = 0; i < hexStr.length(); i++) {
                replacement.append(SECTION).append(Character.toLowerCase(hexStr.charAt(i)));
            }
            matcher.appendReplacement(hexConverted, Matcher.quoteReplacement(replacement.toString()));
        }
        matcher.appendTail(hexConverted);
        String s = hexConverted.toString();

        StringBuilder out = new StringBuilder(s.length());
        for (int i = 0; i < s.length(); i++) {
            char current = s.charAt(i);
            boolean code = current == '&'
                    && i + 1 < s.length()
                    && CODES.indexOf(s.charAt(i + 1)) >= 0;
            out.append(code ? SECTION : current);
        }
        return out.toString();
    }


    /**
     * Обратное к {@link #legacy}: снять цветовые коды.
     *
     * <p>Убирает и обычные {@code §c}, и hex-последовательность
     * {@code §x§f§f§…} — она состоит из тех же пар. Нужно там, где строку кладут
     * в поле, которое legacy не умеет: заголовок скорборда, лог, веб-виджет.
     */
    public static String plain(String text) {
        if (text == null) {
            return "";
        }
        StringBuilder out = new StringBuilder(text.length());
        for (int i = 0; i < text.length(); i++) {
            if (text.charAt(i) == SECTION && i + 1 < text.length()) {
                i++;
                continue;
            }
            out.append(text.charAt(i));
        }
        return out.toString();
    }

    /**
     * Цвет необязателен: без него иконка просто наследует цвет контекста.
     *
     * @return {@code §x§…} либо пустая строка — её и надо подставлять как есть,
     *         тогда отсутствие цвета ничего не ломает в собранной строке
     */
    public static String color(String hex) {
        if (hex == null || !HEX.matcher(hex).matches()) {
            return "";
        }
        StringBuilder out = new StringBuilder(14).append(SECTION).append('x');
        for (int i = 1; i < hex.length(); i++) {
            out.append(SECTION).append(Character.toLowerCase(hex.charAt(i)));
        }
        return out.toString();
    }
}
