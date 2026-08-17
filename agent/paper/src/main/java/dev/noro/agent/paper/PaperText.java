package dev.noro.agent.paper;

import dev.noro.agent.core.TextMarkup;
import dev.noro.agent.core.TextSpan;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.TextComponent;
import net.kyori.adventure.text.event.ClickEvent;
import net.kyori.adventure.text.event.HoverEvent;
import net.kyori.adventure.text.format.TextColor;
import net.kyori.adventure.text.format.TextDecoration;

/**
 * Разметка шаблонов мастера → Adventure-компонент.
 *
 * <p>До этого Paper брал шаблоны через {@code LegacyComponentSerializer}, а он
 * знает только {@code §}-коды: тексты с веб-палитрой (`#f87171`) уезжали в чат
 * вместе с решёткой, а ссылки на свод не были кликабельными. Разбор общий с
 * модами и живёт в core — иначе один шаблон выглядел бы на платформах по-разному.
 */
final class PaperText {

    private PaperText() {}

    static Component parse(String message) {
        TextComponent.Builder root = Component.text();
        for (TextSpan span : TextMarkup.parse(message)) {
            root.append(component(span));
        }
        return root.build();
    }

    private static Component component(TextSpan span) {
        TextComponent.Builder piece = Component.text()
                .content(span.text())
                .decoration(TextDecoration.BOLD, span.bold())
                .decoration(TextDecoration.ITALIC, span.italic())
                .decoration(TextDecoration.UNDERLINED, span.underlined() || span.linked())
                .decoration(TextDecoration.STRIKETHROUGH, span.strikethrough())
                .decoration(TextDecoration.OBFUSCATED, span.obfuscated());
        if (span.hasColor()) {
            piece.color(TextColor.color(span.color()));
        }
        if (span.linked()) {
            piece.clickEvent(ClickEvent.openUrl(span.url()))
                    .hoverEvent(HoverEvent.showText(Component.text(span.url())));
        }
        return piece.build();
    }
}
