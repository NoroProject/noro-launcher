package dev.noro.client.ui;

import java.util.List;
import java.util.function.BiConsumer;
import net.minecraft.client.gui.GuiGraphics;

/**
 * Список строк с выделением и клавиатурой.
 *
 * <p>Тот же список, что в лаунчере: очередь дел, свод правил, свои наказания —
 * это одна и та же таблица с разным содержимым строки. Рисует строку тот, кто
 * знает данные; список отвечает за прокрутку, выделение и попадание мышью.
 *
 * @param <T> что в строках
 */
public final class RowList<T> {

    /** Высота строки: две сетки текста плюс воздух. */
    public static final int ROW = 4 * Theme.GRID;

    private final BiConsumer<Row, T> painter;
    private List<T> items = List.of();
    private int selected;
    private int scroll;

    /**
     * Тянуть ли список к выбранной строке.
     *
     * <p>Раньше он тянулся всегда, каждый кадр, — и колесо мыши упиралось в
     * выбранное правило: список отматывался и тут же прыгал назад, будто дальше
     * ничего нет. Подтягивать нужно ровно тогда, когда выбор сменили с
     * клавиатуры и он мог уехать за край; при прокрутке мышью человек сам
     * решает, куда смотреть.
     */
    private boolean follow = true;
    private int x;
    private int y;
    private int w;
    private int rows;

    /** Что доступно художнику строки. */
    public record Row(GuiGraphics g, int x, int y, int w, boolean active, boolean hovered) {}

    public RowList(BiConsumer<Row, T> painter) {
        this.painter = painter;
    }

    public void items(List<T> items) {
        this.items = items == null ? List.of() : items;
        if (selected >= this.items.size()) {
            selected = Math.max(0, this.items.size() - 1);
        }
    }

    public T selected() {
        return selected < items.size() ? items.get(selected) : null;
    }

    public boolean empty() {
        return items.isEmpty();
    }

    public void render(GuiGraphics g, int x, int y, int w, int h, int mouseX, int mouseY) {
        this.x = x;
        this.y = y;
        this.w = w;
        this.rows = Math.max(1, h / ROW);
        if (follow) {
            keepVisible();
            follow = false;
        }
        scroll = Math.max(0, Math.min(scroll, Math.max(0, items.size() - rows)));

        for (int i = 0; i < rows && scroll + i < items.size(); i++) {
            int index = scroll + i;
            int top = y + i * ROW;
            boolean hovered = mouseX >= x && mouseX <= x + w && mouseY >= top && mouseY < top + ROW;
            boolean active = index == selected;
            if (active) {
                g.fill(x, top, x + w, top + ROW - 1, Theme.BG_CARD_HOVER);
                Surface.marker(g, x, top, ROW - 1, Theme.CTA);
            } else if (hovered) {
                g.fill(x, top, x + w, top + ROW - 1, Theme.BG_CARD);
            }
            painter.accept(new Row(g, x + 2 * Theme.GRID, top + Theme.GRID, w - 3 * Theme.GRID, active, hovered), items.get(index));
        }
        scrollbar(g, h);
    }

    /** Полоса прокрутки появляется, только когда есть что прокручивать. */
    private void scrollbar(GuiGraphics g, int h) {
        if (items.size() <= rows) {
            return;
        }
        int trackX = x + w - 2;
        int thumb = Math.max(4 * Theme.GRID, h * rows / items.size());
        int offset = (h - thumb) * scroll / Math.max(1, items.size() - rows);
        g.fill(trackX, y, trackX + 2, y + h, Theme.BG_INPUT);
        g.fill(trackX, y + offset, trackX + 2, y + offset + thumb, Theme.TEXT_MUTED);
    }

    private void keepVisible() {
        if (selected < scroll) {
            scroll = selected;
        } else if (selected >= scroll + rows) {
            scroll = selected - rows + 1;
        }
    }

    /** {@code true} — попали в строку и выделение сменилось. */
    public boolean click(double mouseX, double mouseY) {
        if (mouseX < x || mouseX > x + w || mouseY < y) {
            return false;
        }
        int row = (int) ((mouseY - y) / ROW);
        if (row < 0 || row >= rows || scroll + row >= items.size()) {
            return false;
        }
        // Щелчка нет намеренно: модератор в ванише не должен выдавать себя
        // звуком, а игроку лишний клик в списке ничего не сообщает.
        selected = scroll + row;
        return true;
    }

    public void scroll(double delta) {
        scroll = Math.max(0, Math.min(scroll - (int) delta, Math.max(0, items.size() - rows)));
    }

    public void move(int step) {
        follow = true;
        if (!items.isEmpty()) {
            selected = Math.max(0, Math.min(selected + step, items.size() - 1));
        }
    }
}
