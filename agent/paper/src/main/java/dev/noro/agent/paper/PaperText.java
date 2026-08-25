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
        if (span.hasFont()) {
            // Шрифт куска: им плашка роли отличается от обычного текста. Кривое
            // имя не должно ронять всё сообщение — тогда кусок остаётся обычным.
            try {
                piece.font(net.kyori.adventure.key.Key.key(span.font()));
            } catch (RuntimeException ignored) {
                // Имя не разобралось — рисуем обычным шрифтом.
            }
        }
        if (span.hasColor()) {
            piece.color(TextColor.color(span.color()));
        }
        if (span.linked()) {
            // `cmd:` — кнопка меню разбора: клик выполняет команду. Отдельного
            // поля в TextSpan для этого нет намеренно: схема в ссылке понятна
            // и на той платформе, которая про меню ничего не знает.
            String url = span.url();
            if (url.startsWith("cmd:")) {
                String command = url.substring(4);
                piece.clickEvent(ClickEvent.runCommand(command))
                        .hoverEvent(HoverEvent.showText(Component.text(command)));
            } else {
                piece.clickEvent(ClickEvent.openUrl(url))
                        .hoverEvent(HoverEvent.showText(Component.text(url)));
            }
        }
        return piece.build();
    }
}
