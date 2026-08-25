package dev.noro.client.staff.ui;

import dev.noro.client.staff.Vanish;
import dev.noro.client.ui.Hud;
import dev.noro.client.ui.Surface;
import dev.noro.client.ui.Theme;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * «Ты сейчас невидим» — левым нижним углом, над строкой чата.
 *
 * <p>Это единственное состояние модератора, которое меняет смысл всего, что он
 * делает: в ванише его не слышат, к нему не подходят, и разговор с игроком
 * выглядит разговором с пустотой. Сообщение в чат о включении уезжает вверх
 * через полминуты, и дальше вспомнить не по чему.
 *
 * <p>Слева внизу, а не рядом с карточкой дела: ваниш живёт сам по себе, и
 * привязывать его к разбору неправильно — включают его и без всякого дела.
 */
public final class VanishBadge {

    private static final int PAD = 2 * Theme.GRID;

    private VanishBadge() {}

    public static void render(GuiGraphics g, int width, int height) {
        if (!Vanish.self()) {
            return;
        }
        Font font = Theme.font();
        Component label = Component.translatable("noro.staff.vanish");
        int others = Vanish.others();
        Component extra = others == 0 ? null : Component.literal("+" + others);

        int w = PAD + 2 * Theme.GRID + font.width(label) + PAD
                + (extra == null ? 0 : font.width(extra) + 2 * Theme.GRID);
        int h = 5 * Theme.GRID;
        int x = Hud.MARGIN;
        // Над строкой чата: она сама лежит у нижнего края и перекрыла бы значок.
        int y = height - h - Hud.MARGIN - 12 * Theme.GRID;

        Surface.window(g, x, y, w, h);
        Surface.marker(g, x + 2, y + 2, h - 4, Theme.WARNING);

        int text = y + (h - 8) / 2;
        g.drawString(font, label, x + PAD + 2 * Theme.GRID, text, Theme.WARNING, false);
        if (extra != null) {
            // Сколько ещё скрытых видно рядом: в ванише их несколько, и знать,
            // что ты не один, важнее, чем кто именно.
            g.drawString(font, extra, x + w - PAD - font.width(extra), text, Theme.TEXT_MUTED, false);
        }
    }
}
