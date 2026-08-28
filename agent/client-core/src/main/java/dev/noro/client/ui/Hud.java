package dev.noro.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * HUD primitives: launcher-styled overlays drawn on top of the game. Never grabs
 * input.
 *
 * <p>Corners are named rather than given as coordinates so several features
 * drawing at once can agree on placement.
 */
public final class Hud {

    public static final int MARGIN = 2 * Theme.GRID;

    public enum Corner {
        TOP_LEFT,
        TOP_RIGHT,
        BOTTOM_LEFT,
        BOTTOM_RIGHT
    }

    private Hud() {}

    /** Returns the card's top edge, so the next one can stack against it. */
    public static int card(GuiGraphics g, Corner corner, int w, int h, int stackedAbove) {
        Minecraft mc = Minecraft.getInstance();
        int screenW = mc.getWindow().getGuiScaledWidth();
        int screenH = mc.getWindow().getGuiScaledHeight();
        int x = switch (corner) {
            case TOP_LEFT, BOTTOM_LEFT -> MARGIN;
            case TOP_RIGHT, BOTTOM_RIGHT -> screenW - w - MARGIN;
        };
        int y = switch (corner) {
            // Below the potion effect icons — that strip is already taken.
            case TOP_LEFT, TOP_RIGHT -> MARGIN + 6 * Theme.GRID + stackedAbove;
            case BOTTOM_LEFT, BOTTOM_RIGHT -> screenH - h - MARGIN - stackedAbove;
        };
        Surface.card(g, x, y, w, h);
        return y;
    }

    public static void row(GuiGraphics g, int x, int y, int w, Component label, Component value, int valueColor) {
        Minecraft mc = Minecraft.getInstance();
        g.drawString(mc.font, label, x, y, Theme.TEXT_MUTED);
        int valueWidth = mc.font.width(value);
        g.drawString(mc.font, value, x + w - valueWidth, y, valueColor);
    }

    /** Progress bar. {@code fraction} is clamped to 0..1. */
    public static void bar(GuiGraphics g, int x, int y, int w, double fraction, int color) {
        int h = Theme.GRID;
        g.fill(x, y, x + w, y + h, Theme.BG_INPUT);
        int filled = (int) (w * Math.max(0, Math.min(1, fraction)));
        if (filled > 0) {
            g.fill(x, y, x + filled, y + h, color);
        }
    }
}
