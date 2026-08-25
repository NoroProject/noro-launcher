package dev.noro.client.ui;

import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

/**
 * Основа экрана в стиле лаунчера: подложка, заголовок, рабочая область.
 *
 * <p>Здесь живёт вся рамка, чтобы экраны модов её не переписывали: каждый
 * получает готовое окно и заполняет только середину. Без этого «в стиле Noro»
 * продержалось бы ровно до третьего экрана.
 */
public abstract class NoroScreen extends Screen {

    /** Ширина окна в шагах сетки. Дальше растягивать некуда — читать неудобно. */
    private static final int MAX_W = 112;

    private static final int MAX_H = 76;

    protected int windowX;
    protected int windowY;
    protected int windowW;
    protected int windowH;

    /**
     * Рабочая область — то же, что приходит в {@link #content}, но известное уже
     * при расстановке виджетов.
     *
     * <p>Без этих полей экран считал одно и то же дважды: виджеты — отступами от
     * края окна в {@code layout}, надписи и списки — отступами от начала области
     * в {@code content}. Две системы координат для одной колонки расходились на
     * каждой правке, и чипы наказаний налезали на список правил.
     */
    protected int contentX;
    protected int contentY;
    protected int contentW;
    protected int contentH;

    protected NoroScreen(Component title) {
        super(title);
    }

    /** Подпись под заголовком: одна строка о том, что это за экран. */
    protected Component subtitle() {
        return null;
    }

    /** Содержимое окна. Координаты уже внутри рамки. */
    protected abstract void content(GuiGraphics g, int x, int y, int w, int h, int mouseX, int mouseY);

    /**
     * Сколько по высоте нужно самому содержимому.
     *
     * <p>Экран, которому нужно меньше, столько и займёт. Окно во весь рост с
     * двумя строками списка и пустотой на две трони выглядит не «просторно», а
     * недогруженным: глаз ищет то, чего там нет.
     */
    protected int contentHeight() {
        return MAX_H * Theme.GRID;
    }

    /** Высота нижней полосы с кнопками. Ноль — полосы нет. */
    protected int footerHeight() {
        return 0;
    }

    /** Верх кнопок нижнего ряда: их место считает основа, а не каждый экран. */
    protected int footerY;

    /** Размер окна, под который расставлены виджеты. */
    private int laidOutFor;

    @Override
    protected void init() {
        measure();
        laidOutFor = windowH;
        layout();
    }

    /**
     * Посчитать окно и рабочую область.
     *
     * <p>Зовётся и при сборке экрана, и перед каждой отрисовкой: и подпись под
     * заголовком, и высота содержимого зависят от данных, а они догружаются
     * кадром. Пересчёт дешевле, чем расхождение между тем, где надпись
     * нарисована, и тем, где её ждут.
     */
    private void measure() {
        int pad = 4 * Theme.GRID;
        int header = pad + 3 * Theme.GRID + (subtitle() == null ? 0 : 4 * Theme.GRID) + 2 * Theme.GRID;
        int wanted = header + contentHeight() + footerHeight() + pad;

        windowW = Math.min(width - 8 * Theme.GRID, MAX_W * Theme.GRID);
        windowH = Math.min(height - 8 * Theme.GRID, Math.min(MAX_H * Theme.GRID, wanted));
        windowX = (width - windowW) / 2;
        windowY = (height - windowH) / 2;

        contentX = windowX + pad;
        contentY = windowY + header;
        contentW = windowW - 2 * pad;
        footerY = windowY + windowH - pad - NoroButton.HEIGHT;
        contentH = windowY + windowH - pad - contentY - footerHeight();
    }

    /** Виджеты расставляются здесь: размеры окна уже посчитаны. */
    protected void layout() {}

    /**
     * Всё, что позади виджетов: затемнение, рамка окна, шапка и содержимое.
     *
     * <p>Рисуется именно здесь, а не в {@code render}: ванильный
     * {@code Screen.render} первым делом зовёт {@code renderBackground}, и
     * панель, нарисованная до него, оказывалась бы под ним. Кнопки при этом
     * ложатся сверху сами — их рисует базовый класс следом.
     *
     * <p>{@code super.renderBackground} не зовём намеренно: он включает
     * ванильное размытие, а оно постпроцесс поверх кадра — размывало бы не мир
     * за панелью, а саму панель вместе с текстом.
     */
    @Override
    public void renderBackground(GuiGraphics g, int mouseX, int mouseY, float partial) {
        measure();
        if (windowH != laidOutFor) {
            // Содержимого стало больше или меньше — виджеты, расставленные под
            // прежний размер, оказались бы не на своих местах.
            laidOutFor = windowH;
            clearWidgets();
            layout();
        }
        g.fill(0, 0, width, height, Theme.OVERLAY);
        Surface.window(g, windowX, windowY, windowW, windowH);
        header(g);
        footer(g);
        content(g, contentX, contentY, contentW, contentH, mouseX, mouseY);
    }

    private void header(GuiGraphics g) {
        int pad = 4 * Theme.GRID;
        Surface.header(g, windowX, windowY, windowW, contentY - 2 * Theme.GRID, pad);
        g.drawString(font, title, windowX + pad + Theme.GRID, windowY + pad, Theme.TEXT, false);
        Component subtitle = subtitle();
        if (subtitle != null) {
            g.drawString(font, subtitle, windowX + pad + Theme.GRID,
                    windowY + pad + 3 * Theme.GRID, Theme.TEXT_MUTED, false);
        }
    }

    private void footer(GuiGraphics g) {
        if (footerHeight() > 0) {
            Surface.footer(g, windowX, windowY + windowH, windowW, footerY - 2 * Theme.GRID);
        }
    }

    /**
     * Экран не ставит игру на паузу: в мультиплеере это ничего не даёт, а
     * открытый GUI и так значит «я не двигаюсь».
     */
    @Override
    public boolean isPauseScreen() {
        return false;
    }
}
