package dev.noro.agent.mod;

import dev.noro.agent.core.TextMarkup;
import dev.noro.agent.core.TextSpan;
import net.minecraft.network.chat.ClickEvent;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.HoverEvent;
import net.minecraft.network.chat.MutableComponent;
import net.minecraft.network.chat.Style;
import net.minecraft.network.chat.TextColor;
import net.minecraft.server.level.ServerPlayer;

/**
 * Разметка шаблонов мастера → ванильный {@code Component}.
 */
final class ModText {

    private ModText() {}

    static Component parse(String message) {
        //#if MC>=11900
        MutableComponent root = Component.empty();
        //#else
        //$$ MutableComponent root = new net.minecraft.network.chat.TextComponent("");
        //#endif
        for (TextSpan span : TextMarkup.parse(message)) {
            root.append(component(span));
        }
        return root;
    }

    private static Component component(TextSpan span) {
        //#if MC>=11900
        MutableComponent piece = Component.literal(span.text());
        //#else
        //$$ MutableComponent piece = new net.minecraft.network.chat.TextComponent(span.text());
        //#endif
        return piece.setStyle(style(span));
    }

    private static Style style(TextSpan span) {
        Style style = Style.EMPTY
                .withBold(span.bold())
                .withItalic(span.italic())
                .withUnderlined(span.underlined() || span.linked())
                .withStrikethrough(span.strikethrough())
                .withObfuscated(span.obfuscated());
        if (span.hasColor()) {
            style = style.withColor(TextColor.fromRgb(span.color()));
        }
        if (span.hasFont()) {
            style = withFont(style, span.font());
        }
        return span.linked() ? withLink(style, span.url()) : style;
    }

    /**
     * Шрифт куска: им плашка роли отличается от обычного текста.
     *
     * <p>Кривое имя шрифта роняло бы разбор целиком, а с ним и всё сообщение,
     * поэтому такой кусок просто остаётся обычным.
     */
    private static Style withFont(Style base, String font) {
        try {
            // Два независимых разрыва: с 1.21.9 шрифт описывается не
            // идентификатором, а FontDescription; с 1.21.11 сам идентификатор
            // переименован в Identifier.
            //#if MC>=12111
            //$$ return base.withFont(new net.minecraft.network.chat.FontDescription.Resource(
            //$$         net.minecraft.resources.Identifier.tryParse(font)));
            //#elseif MC>=12109
            //$$ return base.withFont(new net.minecraft.network.chat.FontDescription.Resource(
            //$$         net.minecraft.resources.ResourceLocation.tryParse(font)));
            //#else
            return base.withFont(net.minecraft.resources.ResourceLocation.tryParse(font));
            //#endif
        } catch (RuntimeException e) {
            return base;
        }
    }

    private static Style withLink(Style base, String url) {
        // `cmd:` — кнопка меню разбора: клик выполняет команду, а не открывает
        // адрес. Схема живёт в самой ссылке, чтобы core не знал про платформы.
        if (url.startsWith("cmd:")) {
            return withCommand(base, url.substring(4));
        }
        //#if MC>=12105
        //$$ return base.withClickEvent(new ClickEvent.OpenUrl(java.net.URI.create(url)))
        //$$         .withHoverEvent(new HoverEvent.ShowText(Component.literal(url)));
        //#elseif MC>=11900
        return base.withClickEvent(new ClickEvent(ClickEvent.Action.OPEN_URL, url))
                .withHoverEvent(new HoverEvent(HoverEvent.Action.SHOW_TEXT, Component.literal(url)));
        //#else
        //$$ return base.withClickEvent(new ClickEvent(ClickEvent.Action.OPEN_URL, url))
        //$$         .withHoverEvent(new HoverEvent(
        //$$                 HoverEvent.Action.SHOW_TEXT, new net.minecraft.network.chat.TextComponent(url)));
        //#endif
    }

    private static Style withCommand(Style base, String command) {
        //#if MC>=12105
        //$$ return base.withClickEvent(new ClickEvent.RunCommand(command))
        //$$         .withHoverEvent(new HoverEvent.ShowText(Component.literal(command)));
        //#elseif MC>=11900
        return base.withClickEvent(new ClickEvent(ClickEvent.Action.RUN_COMMAND, command))
                .withHoverEvent(new HoverEvent(HoverEvent.Action.SHOW_TEXT, Component.literal(command)));
        //#else
        //$$ return base.withClickEvent(new ClickEvent(ClickEvent.Action.RUN_COMMAND, command))
        //$$         .withHoverEvent(new HoverEvent(
        //$$                 HoverEvent.Action.SHOW_TEXT, new net.minecraft.network.chat.TextComponent(command)));
        //#endif
    }

    static void send(ServerPlayer player, Component component) {
        //#if MC>=11900
        player.sendSystemMessage(component);
        //#else
        //$$ player.sendMessage(component, net.minecraft.Util.NIL_UUID);
        //#endif
    }

    static void send(ServerPlayer player, String message) {
        send(player, parse(message));
    }

    static Component translatable(String key, Object... args) {
        //#if MC>=11900
        return Component.translatable(key, args);
        //#else
        //$$ return new net.minecraft.network.chat.TranslatableComponent(key, args);
        //#endif
    }
}
