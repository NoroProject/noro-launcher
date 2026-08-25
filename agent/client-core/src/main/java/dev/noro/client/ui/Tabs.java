package dev.noro.client.ui;

import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Полоса вкладок: подчёркивание, а не рамка.
 *
 * <p>Так же, как в лаунчере и на сайте: активная вкладка отмечена линией снизу
 * и цветом текста. Рамки вокруг вкладок делают из шапки таблицу, а это не
 * таблица.
 */
public final class Tabs {

    public static final int HEIGHT = 6 * Theme.GRID;

    private final List<Component> titles;
    private int active;
    private int x;
    private int y;
    private int[] widths;

    public Tabs(List<Component> titles) {
        this.titles = titles;
        this.widths = new int[titles.size()];
    }

    public int active() {
        return active;
    }

    public void active(int index) {
        if (index >= 0 && index < titles.size()) {
            active = index;
        }
    }

    public void render(GuiGraphics g, int x, int y, int w, int mouseX, int mouseY) {
        this.x = x;
        this.y = y;
        Minecraft mc = Minecraft.getInstance();
        int cursor = x;
        for (int i = 0; i < titles.size(); i++) {
            Component title = titles.get(i);
            int tabW = mc.font.width(title) + 4 * Theme.GRID;
            widths[i] = tabW;
            boolean hovered = mouseX >= cursor && mouseX < cursor + tabW
                    && mouseY >= y && mouseY < y + HEIGHT;
            int color = i == active ? Theme.TEXT : hovered ? Theme.TEXT_SECONDARY : Theme.TEXT_MUTED;
            g.drawCenteredString(
                    mc.font, title, cursor + tabW / 2, y + (HEIGHT - 8) / 2 - 1, color);
            if (i == active) {
                g.fill(cursor, y + HEIGHT - 2, cursor + tabW, y + HEIGHT, Theme.CTA);
            }
            cursor += tabW;
        }
        // Линия под всей полосой: она отделяет шапку от содержимого.
        g.fill(x, y + HEIGHT - 1, x + w, y + HEIGHT, Theme.BORDER);
    }

    /** {@code true} — попали во вкладку и она сменилась. */
    public boolean click(double mouseX, double mouseY) {
        if (mouseY < y || mouseY >= y + HEIGHT) {
            return false;
        }
        int cursor = x;
        for (int i = 0; i < titles.size(); i++) {
            if (mouseX >= cursor && mouseX < cursor + widths[i]) {
                boolean changed = active != i;
                active = i;
                return changed;
            }
            cursor += widths[i];
        }
        return false;
    }
}
