package dev.noro.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/** The in-game equivalent of {@code noro-panel} on the site. */
public final class Card {

    public static final int PAD = 2 * Theme.GRID;

    private Card() {}

    /** Returns the top of the content area, below the header. */
    public static int titled(GuiGraphics g, int x, int y, int w, int h, Component title) {
        Surface.card(g, x, y, w, h);
        g.drawString(
                Minecraft.getInstance().font,
                title,
                x + PAD,
                y + PAD,
                Theme.TEXT_MUTED);
        return y + PAD + 3 * Theme.GRID;
    }

    public static int plain(GuiGraphics g, int x, int y, int w, int h) {
        Surface.card(g, x, y, w, h);
        return y + PAD;
    }

    /** A number with a caption under it, as in the admin summary tiles. */
    public static void metric(GuiGraphics g, int x, int y, int w, Component value, Component label) {
        Minecraft mc = Minecraft.getInstance();
        g.drawString(mc.font, value, x, y, Theme.TEXT);
        g.drawString(mc.font, label, x, y + 3 * Theme.GRID, Theme.TEXT_MUTED);
    }
}
