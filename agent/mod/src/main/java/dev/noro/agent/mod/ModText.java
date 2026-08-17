package dev.noro.agent.mod;

import dev.noro.agent.core.TextMarkup;
import dev.noro.agent.core.TextSpan;
import net.minecraft.network.chat.ClickEvent;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.HoverEvent;
import net.minecraft.network.chat.MutableComponent;
import net.minecraft.network.chat.Style;
import net.minecraft.network.chat.TextColor;

/**
 * Разметка шаблонов мастера → ванильный {@code Component}.
 *
 * <p>Сам разбор живёт в core ({@link TextMarkup}), здесь только сборка
 * компонентов: цвета, стили и кликабельные ссылки. Так Paper и моды показывают
 * один шаблон одинаково — раньше разбор был только здесь, и на Paper те же
 * тексты уезжали в чат вместе с «#f87171».
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
        return span.linked() ? withLink(style, span.url()) : style;
    }

    /**
     * Кликабельная ссылка. Единственный разрыв API на диапазоне: в 1.21.5
     * события стали sealed-типами вместо пары «действие + строка».
     */
    private static Style withLink(Style base, String url) {
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
}
