package dev.noro.client.staff.ui;

import dev.noro.client.staff.CaseState;
import dev.noro.client.ui.Surface;
import dev.noro.client.ui.Theme;
import dev.noro.client.staff.CaseFrames;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Отказ мастера — единственная проверка, которой стоит верить.
 *
 * <p>Права в {@code Ready} только гасят кнопки; всё, что до кнопки дошло,
 * решается на той стороне. Поэтому отказ показывается прямо на карточке, а не
 * молча пропадает.
 */
final class Rejection {

    private Rejection() {}

    static void render(GuiGraphics g, Font font, CaseState state, int width, int height) {
        CaseFrames.Rejected rejected = state.rejected();
        if (rejected == null) {
            return;
        }
        int w = 60 * Theme.GRID;
        int h = 9 * Theme.GRID;
        int x = (width - w) / 2;
        int y = height - 18 * Theme.GRID;
        Surface.card(g, x, y, w, h);
        g.drawString(
                font,
                Component.translatable(rejected.reason()),
                x + 3 * Theme.GRID,
                y + 2 * Theme.GRID,
                Theme.ERROR);
        // Номер рядом с намерением: «Punish · 1305» достаточно, чтобы назвать
        // отказ, не пересказывая переведённый текст.
        Component footer = rejected.number() > 0
                ? Component.translatable(
                        "noro.cases.rejected.intent-numbered", rejected.intent(), rejected.number())
                : Component.translatable("noro.cases.rejected.intent", rejected.intent());
        g.drawString(font, footer, x + 3 * Theme.GRID, y + 5 * Theme.GRID + 1, Theme.TEXT_MUTED);
    }
}
