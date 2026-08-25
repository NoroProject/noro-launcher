package dev.noro.client.ui;

import net.minecraft.client.gui.GuiGraphics;

/**
 * Подложки: окно, карточка, рамка, разделитель.
 *
 * <p>Отдельно от {@link Theme}, потому что тема — это краски и шрифт, а здесь их
 * укладывают слоями. Вместе получался файл, в котором не найти ни палитру, ни
 * отрисовку.
 */
public final class Surface {

    private Surface() {}

    /** Карточка: заливка и контур в одну линию, без отдельной текстуры. */
    public static void card(GuiGraphics g, int x, int y, int w, int h) {
        g.fill(x, y, x + w, y + h, Theme.BG_CARD);
        outline(g, x, y, w, h, Theme.BORDER);
    }

    /**
     * Окно: тень, двойная рамка, заливка.
     *
     * <p>Одна заливка с линией в пиксель — это прямоугольник, а не окно: у него
     * нет ни толщины, ни верха, ни низа, и панель выглядит наклейкой поверх
     * игры. Три слоя дают край: тёмный контур снаружи, светлый блик изнутри,
     * заливка под ними.
     *
     * <p>Кремовых насечек по углам здесь больше нет. Они задумывались приметой
     * лаунчера, а на деле читались как случайные чёрточки: рамка и без них
     * очерчена, а украшение, которое приходится объяснять, — не украшение.
     * Форму окну задают полосы шапки и подвала, а не метки по углам.
     */
    public static void window(GuiGraphics g, int x, int y, int w, int h) {
        g.fill(x + 3, y + 3, x + w + 3, y + h + 3, Theme.SHADOW);
        g.fill(x, y, x + w, y + h, Theme.BG_PANEL);
        outline(g, x, y, w, h, Theme.BORDER_STRONG);
        outline(g, x + 1, y + 1, w - 2, h - 2, Theme.EDGE);
    }

    /** Контур в одну линию: рамка кнопки, поля и карточки — одна и та же. */
    public static void outline(GuiGraphics g, int x, int y, int w, int h, int color) {
        g.fill(x, y, x + w, y + 1, color);
        g.fill(x, y + h - 1, x + w, y + h, color);
        g.fill(x, y, x + 1, y + h, color);
        g.fill(x + w - 1, y, x + w, y + h, color);
    }

    /**
     * Полоса шапки: заливка, линия снизу и метка слева от заголовка.
     *
     * <p>Заголовок сам по себе шапкой не выглядит — он просто первая строка
     * сверху. Метка ростом со строку, а не во всю высоту: полоса от края до края
     * читалась как полоса прокрутки, приехавшая не туда.
     */
    public static void header(GuiGraphics g, int x, int y, int w, int bottom, int pad) {
        g.fill(x + 2, y + 2, x + w - 2, bottom, Theme.BG_CARD);
        g.fill(x + 2, bottom, x + w - 2, bottom + 1, Theme.BORDER_STRONG);
        marker(g, x + pad - Theme.GRID, y + pad - 1, 2 * Theme.GRID, Theme.CTA);
    }

    /**
     * Полоса под кнопками нижнего ряда.
     *
     * <p>Без неё кнопки висят в пустоте посреди панели, и непонятно, относятся
     * они к последнему блоку содержимого или ко всему экрану. Полоса отвечает:
     * ко всему — это подвал.
     */
    public static void footer(GuiGraphics g, int x, int y, int w, int top) {
        g.fill(x + 2, top, x + w - 2, y - 2, Theme.BG_CARD);
        g.fill(x + 2, top, x + w - 2, top + 1, Theme.BORDER_STRONG);
    }

    /** Разделитель внутри карточки: им режут плашку на смысловые куски. */
    public static void divider(GuiGraphics g, int x, int y, int w) {
        g.fill(x, y, x + w, y + 1, Theme.BORDER);
    }

    /** Полоска-акцент слева от строки: ею отмечают выбранное и важное. */
    public static void marker(GuiGraphics g, int x, int y, int h, int color) {
        g.fill(x, y, x + 2, y + h, color);
    }
}
