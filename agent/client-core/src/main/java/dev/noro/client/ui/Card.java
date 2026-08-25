package dev.noro.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Карточка с заголовком: тот же блок, что {@code noro-panel} на сайте.
 *
 * <p>Из карточек собран весь интерфейс лаунчера — сервер, сборка, наказание. В
 * игре они те же: заголовок мелким приглушённым, содержимое под ним.
 */
public final class Card {

    /** Отступ внутри карточки. */
    public static final int PAD = 2 * Theme.GRID;

    private Card() {}

    /**
     * Нарисовать карточку с подписью. Возвращает верх содержимого — под шапкой,
     * чтобы вызывающему не пересчитывать отступ самому.
     */
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

    /** Карточка без подписи — просто фон под содержимое. */
    public static int plain(GuiGraphics g, int x, int y, int w, int h) {
        Surface.card(g, x, y, w, h);
        return y + PAD;
    }

    /**
     * Метрика: крупное число и подпись под ним. Из таких собраны сводки в
     * админке — «жалоб», «человек», «наказаний».
     */
    public static void metric(GuiGraphics g, int x, int y, int w, Component value, Component label) {
        Minecraft mc = Minecraft.getInstance();
        g.drawString(mc.font, value, x, y, Theme.TEXT);
        g.drawString(mc.font, label, x, y + 3 * Theme.GRID, Theme.TEXT_MUTED);
    }
}
