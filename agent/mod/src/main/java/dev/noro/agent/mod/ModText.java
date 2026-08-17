package dev.noro.agent.mod;

import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.ClickEvent;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.HoverEvent;
import net.minecraft.network.chat.MutableComponent;
import net.minecraft.network.chat.Style;
import net.minecraft.network.chat.TextColor;

/**
 * Разбор цветовых кодов и ссылок в компоненты Minecraft.
 *
 * <p>Поддерживает:
 * <ul>
 *   <li>HEX-цвета веба: {@code #f87171} и {@code &#f87171}</li>
 *   <li>Legacy-последовательности: {@code §x§f§8§7§1§7§1}</li>
 *   <li>Форматирование амперсандом и параграфом: {@code &c}, {@code &l}, {@code §c}, {@code §l}</li>
 *   <li>Ссылки: {@code [правило 1.1](https://noro.dalynkaa.dev/rules#rule-1.1)} и сырые {@code https://...}</li>
 * </ul>
 */
final class ModText {

    private ModText() {}

    public static Component parse(String message) {
        return parseSegment(message, Style.EMPTY);
    }

    public static Component parseSegment(String message, Style baseStyle) {
        if (message == null || message.isEmpty()) {
            //#if MC>=11900
            return Component.empty();
            //#else
            //$$ return new net.minecraft.network.chat.TextComponent("");
            //#endif
        }

        //#if MC>=11900
        MutableComponent root = Component.empty();
        //#else
        //$$ MutableComponent root = new net.minecraft.network.chat.TextComponent("");
        //#endif

        Style currentStyle = baseStyle;
        StringBuilder currentText = new StringBuilder();

        int i = 0;
        int len = message.length();
        while (i < len) {
            char c = message.charAt(i);

            // 1. Legacy §x§r§r§g§g§b§b hex sequence
            if (c == '§' && i + 13 < len && Character.toLowerCase(message.charAt(i + 1)) == 'x') {
                boolean validHexSequence = true;
                StringBuilder hexBuf = new StringBuilder();
                for (int h = 0; h < 6; h++) {
                    if (message.charAt(i + 2 + h * 2) == '§') {
                        hexBuf.append(message.charAt(i + 3 + h * 2));
                    } else {
                        validHexSequence = false;
                        break;
                    }
                }
                if (validHexSequence) {
                    if (currentText.length() > 0) {
                        appendChunk(root, currentText.toString(), currentStyle);
                        currentText.setLength(0);
                    }
                    try {
                        int rgb = Integer.parseInt(hexBuf.toString(), 16);
                        currentStyle = baseStyle.withColor(TextColor.fromRgb(rgb));
                    } catch (Exception ignored) {}
                    i += 14;
                    continue;
                }
            }

            // 2. #rrggbb or &#rrggbb
            if ((c == '#' && i + 6 < len) || (c == '&' && i + 7 < len && message.charAt(i + 1) == '#')) {
                int hexStart = (c == '#') ? i + 1 : i + 2;
                int hexLen = 6;
                if (hexStart + hexLen <= len) {
                    String hexCandidate = message.substring(hexStart, hexStart + hexLen);
                    if (hexCandidate.matches("[0-9a-fA-F]{6}")) {
                        if (currentText.length() > 0) {
                            appendChunk(root, currentText.toString(), currentStyle);
                            currentText.setLength(0);
                        }
                        try {
                            int rgb = Integer.parseInt(hexCandidate, 16);
                            currentStyle = baseStyle.withColor(TextColor.fromRgb(rgb));
                        } catch (Exception ignored) {}
                        i = hexStart + hexLen;
                        continue;
                    }
                }
            }

            // 3. Markdown link [label](url)
            if (c == '[' && i < len) {
                int endBracket = message.indexOf(']', i);
                if (endBracket > i && endBracket + 1 < len && message.charAt(endBracket + 1) == '(') {
                    int endParen = message.indexOf(')', endBracket + 1);
                    if (endParen > endBracket + 1) {
                        String label = message.substring(i + 1, endBracket);
                        String url = message.substring(endBracket + 2, endParen);
                        if (currentText.length() > 0) {
                            appendChunk(root, currentText.toString(), currentStyle);
                            currentText.setLength(0);
                        }
                        root.append(parseSegment(label, withLink(currentStyle, url)));
                        i = endParen + 1;
                        continue;
                    }
                }
            }

            // 4. Raw URL http:// or https://
            if ((c == 'h' || c == 'H')
                    && (message.regionMatches(true, i, "http://", 0, 7)
                            || message.regionMatches(true, i, "https://", 0, 8))) {
                int endUrl = i;
                while (endUrl < len && !Character.isWhitespace(message.charAt(endUrl)) && message.charAt(endUrl) != ')') {
                    endUrl++;
                }
                String url = message.substring(i, endUrl);
                if (currentText.length() > 0) {
                    appendChunk(root, currentText.toString(), currentStyle);
                    currentText.setLength(0);
                }
                root.append(parseSegment(url, withLink(currentStyle, url)));
                i = endUrl;
                continue;
            }

            // 5. & or § legacy format codes
            if ((c == '&' || c == '§') && i + 1 < len) {
                char code = Character.toLowerCase(message.charAt(i + 1));
                ChatFormatting format = ChatFormatting.getByCode(code);
                if (format != null) {
                    if (currentText.length() > 0) {
                        appendChunk(root, currentText.toString(), currentStyle);
                        currentText.setLength(0);
                    }
                    if (format == ChatFormatting.RESET) {
                        currentStyle = baseStyle;
                    } else if (format.ordinal() < 16) {
                        currentStyle = baseStyle.withColor(format);
                    } else if (code == 'l') {
                        currentStyle = currentStyle.withBold(true);
                    } else if (code == 'o') {
                        currentStyle = currentStyle.withItalic(true);
                    } else if (code == 'n') {
                        currentStyle = currentStyle.withUnderlined(true);
                    } else if (code == 'm') {
                        currentStyle = currentStyle.withStrikethrough(true);
                    } else if (code == 'k') {
                        currentStyle = currentStyle.withObfuscated(true);
                    }
                    i += 2;
                    continue;
                }
            }

            currentText.append(c);
            i++;
        }

        if (currentText.length() > 0) {
            appendChunk(root, currentText.toString(), currentStyle);
        }

        return root;
    }

    private static Style withLink(Style base, String url) {
        //#if MC>=12105
        //$$ ClickEvent click = new ClickEvent.OpenUrl(java.net.URI.create(url));
        //$$ Component tooltip = Component.literal("§7Открыть ссылку:\n§f" + url);
        //$$ HoverEvent hover = new HoverEvent.ShowText(tooltip);
        //#elseif MC>=11900
        ClickEvent click = new ClickEvent(ClickEvent.Action.OPEN_URL, url);
        Component tooltip = Component.literal("§7Открыть ссылку:\n§f" + url);
        HoverEvent hover = new HoverEvent(HoverEvent.Action.SHOW_TEXT, tooltip);
        //#else
        //$$ ClickEvent click = new ClickEvent(ClickEvent.Action.OPEN_URL, url);
        //$$ Component tooltip = new net.minecraft.network.chat.TextComponent("§7Открыть ссылку:\n§f" + url);
        //$$ HoverEvent hover = new HoverEvent(HoverEvent.Action.SHOW_TEXT, tooltip);
        //#endif
        return base.withClickEvent(click).withHoverEvent(hover).withUnderlined(true);
    }

    private static void appendChunk(MutableComponent root, String text, Style style) {
        //#if MC>=11900
        root.append(Component.literal(text).withStyle(style));
        //#else
        //$$ root.append(new net.minecraft.network.chat.TextComponent(text).setStyle(style));
        //#endif
    }
}
