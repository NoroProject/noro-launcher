package dev.noro.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Примитивы HUD: плашки поверх игры в стиле лаунчера.
 *
 * <p>HUD не отнимает управление и ничем не выдаёт себя: ни звука, ни частиц, ни
 * слота интерфейса. Полноэкранный GUI открывается отдельно и только когда
 * действительно нужен.
 *
 * <p>Углы именованы, а не заданы координатами: когда над игрой рисуют сразу
 * несколько функций — разбор, кошелёк, статус сборки, — им нужно договариваться
 * о местах, и договариваться проще про «правый верх», чем про пиксели.
 */
public final class Hud {

    /** Отступ от края экрана. */
    public static final int MARGIN = 2 * Theme.GRID;

    public enum Corner {
        TOP_LEFT,
        TOP_RIGHT,
        BOTTOM_LEFT,
        BOTTOM_RIGHT
    }

    private Hud() {}

    /** Плашка в углу. Возвращает её верх — под ней можно рисовать следующую. */
    public static int card(GuiGraphics g, Corner corner, int w, int h, int stackedAbove) {
        Minecraft mc = Minecraft.getInstance();
        int screenW = mc.getWindow().getGuiScaledWidth();
        int screenH = mc.getWindow().getGuiScaledHeight();
        int x = switch (corner) {
            case TOP_LEFT, BOTTOM_LEFT -> MARGIN;
            case TOP_RIGHT, BOTTOM_RIGHT -> screenW - w - MARGIN;
        };
        int y = switch (corner) {
            // Сверху — под эффектами зелий и бафами: там уже занято.
            case TOP_LEFT, TOP_RIGHT -> MARGIN + 6 * Theme.GRID + stackedAbove;
            case BOTTOM_LEFT, BOTTOM_RIGHT -> screenH - h - MARGIN - stackedAbove;
        };
        Surface.card(g, x, y, w, h);
        return y;
    }

    /** Строка «подпись — значение»: из них состоит почти любая плашка. */
    public static void row(GuiGraphics g, int x, int y, int w, Component label, Component value, int valueColor) {
        Minecraft mc = Minecraft.getInstance();
        g.drawString(mc.font, label, x, y, Theme.TEXT_MUTED);
        int valueWidth = mc.font.width(value);
        g.drawString(mc.font, value, x + w - valueWidth, y, valueColor);
    }

    /** Полоса: время до конца, прогресс, доля. Пустая — просто рамка. */
    public static void bar(GuiGraphics g, int x, int y, int w, double fraction, int color) {
        int h = Theme.GRID;
        g.fill(x, y, x + w, y + h, Theme.BG_INPUT);
        int filled = (int) (w * Math.max(0, Math.min(1, fraction)));
        if (filled > 0) {
            g.fill(x, y, x + filled, y + h, color);
        }
    }
}
