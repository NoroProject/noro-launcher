package dev.noro.agent.core;

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
    private static final char SECTION = '§';
    private static final String RESET = SECTION + "r";

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

    /** Цвет необязателен: без него иконка просто наследует цвет контекста. */
    private static String color(String hex) {
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
