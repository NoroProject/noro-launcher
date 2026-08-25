package dev.noro.client.staff.ui;

import dev.noro.client.staff.CaseModels;
import dev.noro.client.staff.CaseStyle;
import dev.noro.client.ui.RowList;
import dev.noro.client.ui.Theme;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Таблица очереди: шапка и строка.
 *
 * <p>Отдельно от экрана, потому что экран занят другим — поиском, страницами и
 * тем, что происходит по Enter. Здесь только колонки, и считаются они от ширины,
 * а не от магических отступов.
 */
final class QueueTable {

    /** Доли ширины под колонки: номер, цель, вес. Остаток — «ведёт», справа. */
    private static final float COL_NUMBER = 0.22f;

    private static final float COL_TARGET = 0.28f;

    private QueueTable() {}

    /** Шапка. Без неё три колонки чисел читаются как случайный набор. */
    static void header(GuiGraphics g, Font font, int x, int y, int w) {
        int inner = w - 3 * Theme.GRID;
        g.drawString(
                font,
                Component.translatable("noro.cases.queue.col.case"),
                x + 2 * Theme.GRID,
                y,
                Theme.TEXT_MUTED);
        g.drawString(
                font,
                Component.translatable("noro.cases.queue.col.target"),
                x + 2 * Theme.GRID + (int) (inner * COL_NUMBER),
                y,
                Theme.TEXT_MUTED);
        g.drawString(
                font,
                Component.translatable("noro.cases.queue.col.reports"),
                x + 2 * Theme.GRID + (int) (inner * (COL_NUMBER + COL_TARGET)),
                y,
                Theme.TEXT_MUTED);
        Component held = Component.translatable("noro.cases.queue.col.held");
        g.drawString(font, held, x + inner - font.width(held), y, Theme.TEXT_MUTED);
        g.fill(x, y + 3 * Theme.GRID, x + w, y + 3 * Theme.GRID + 1, Theme.BORDER);
    }

    /** Одна строка очереди. */
    static void row(RowList.Row row, CaseModels.Brief c) {
        Font font = Minecraft.getInstance().font;
        int w = row.w();
        row.g().drawString(font, c.label(), row.x(), row.y(), Theme.TEXT_MUTED);

        String target = c.target_name() == null ? "—" : c.target_name();
        row.g().drawString(font, target, row.x() + (int) (w * COL_NUMBER), row.y(), Theme.TEXT);

        row.g()
                .drawString(
                        font,
                        CaseStyle.reports(c.reports_count(), c.reporters_count()),
                        row.x() + (int) (w * (COL_NUMBER + COL_TARGET)),
                        row.y(),
                        Theme.CTA);

        String holder = c.claimed_by_name() == null ? "" : c.claimed_by_name();
        row.g()
                .drawString(
                        font,
                        holder,
                        row.x() + w - font.width(holder),
                        row.y(),
                        CaseStyle.statusColor(c.status()));
    }
}
