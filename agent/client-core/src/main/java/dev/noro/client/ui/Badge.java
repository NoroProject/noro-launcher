package dev.noro.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Бейдж: короткое слово о состоянии — «открыто», «в работе», «истёк».
 *
 * <p>Цветом, а не значком: значок надо выучить, а слово читается сразу. Тон
 * берётся из темы, поэтому «плохо» везде одного оттенка.
 */
public final class Badge {

    public static final int HEIGHT = 4 * Theme.GRID;

    public enum Tone {
        NEUTRAL,
        INFO,
        GOOD,
        WARN,
        BAD
    }

    private Badge() {}

    /** Возвращает ширину — по ней выравнивают то, что идёт следом. */
    public static int draw(GuiGraphics g, int x, int y, Component text, Tone tone) {
        Minecraft mc = Minecraft.getInstance();
        int w = mc.font.width(text) + 3 * Theme.GRID;
        int color = color(tone);
        g.fill(x, y, x + w, y + HEIGHT, Theme.BG_INPUT);
        Surface.outline(g, x, y, w, HEIGHT, color);
        g.drawString(mc.font, text, x + Theme.GRID + 2, y + (HEIGHT - 8) / 2, color, false);
        return w;
    }

    private static int color(Tone tone) {
        return switch (tone) {
            case INFO -> Theme.BLUE;
            case GOOD -> Theme.SUCCESS;
            case WARN -> Theme.WARNING;
            case BAD -> Theme.ERROR;
            case NEUTRAL -> Theme.TEXT_MUTED;
        };
    }
}
