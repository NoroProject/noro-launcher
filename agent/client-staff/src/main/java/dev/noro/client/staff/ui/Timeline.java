package dev.noro.client.staff.ui;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import dev.noro.client.staff.CaseModels;
import dev.noro.client.ui.Theme;
import java.util.List;
import java.util.Set;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Лента разбора: что делали с делом, когда и откуда.
 *
 * <p>Источник подписан словом, а не значком: спор «я этого не делал» решается
 * тем, видно ли, что телепорт пришёл из игры, а вердикт — с сайта.
 */
final class Timeline {

    /** Вердикт, наказание и жалоба — итог разбора, их видно сразу. */
    private static final Set<String> LOUD = Set.of("verdict", "punishment", "report_added");

    private static final int ROW = 10;

    private Timeline() {}

    static void render(
            GuiGraphics g, Font font, List<CaseModels.Event> events, int x, int y, int w, int bottom) {
        g.drawString(font, Component.translatable("noro.cases.card.timeline"), x, y, Theme.TEXT_MUTED);
        if (events == null || events.isEmpty()) {
            return;
        }
        int top = y + 6 * Theme.GRID;
        int rows = (bottom - top) / ROW;
        int from = Math.max(0, events.size() - rows);
        for (int i = from; i < events.size(); i++) {
            row(g, font, events.get(i), x, top + (i - from) * ROW, w);
        }
    }

    private static void row(GuiGraphics g, Font font, CaseModels.Event e, int x, int y, int w) {
        int color = LOUD.contains(e.kind()) ? Theme.TEXT : Theme.TEXT_MUTED;
        g.drawString(font, clock(e.at()), x, y, Theme.TEXT_MUTED);

        Component label = Component.translatable("noro.cases.event." + e.kind());
        int labelX = x + 9 * Theme.GRID;
        g.drawString(font, label, labelX, y, color);

        // Подпись начинается за названием события, а не на фиксированном
        // отступе: «Запрошена проверка клиента» длиннее любой колонки, и
        // подробность налезала прямо на неё.
        String detail = detail(e);
        if (detail.isEmpty()) {
            return;
        }
        int detailX = labelX + font.width(label) + 2 * Theme.GRID;
        int room = x + w - detailX;
        if (room < 8 * Theme.GRID) {
            return;
        }
        g.drawString(font, font.plainSubstrByWidth(detail, room), detailX, y, Theme.TEXT_MUTED);
    }

    /** Из ISO-8601 нужно только время: по нему виден темп разбора. */
    static String clock(String at) {
        if (at == null || at.length() < 16) {
            return "--:--";
        }
        return at.substring(11, 16);
    }

    /**
     * Подпись под событием. {@code payload} у каждого вида свой, поэтому разбор
     * идёт по виду, а не по общей форме — общей у них нет.
     */
    private static String detail(CaseModels.Event e) {
        JsonObject p = e.payload();
        if (p == null) {
            return "";
        }
        return switch (e.kind()) {
            case "note" -> text(p, "text");
            case "punishment" -> text(p, "kind") + " · " + text(p, "reason");
            case "verdict" -> text(p, "verdict") + " · " + text(p, "resolution");
            case "report_added" -> text(p, "reason");
            case "quote" -> text(p, "content");
            case "screenshot" -> text(p, "note");
            case "teleport" -> text(p, "to");
            default -> "";
        };
    }

    private static String text(JsonObject payload, String field) {
        JsonElement value = payload.get(field);
        return value == null || value.isJsonNull() ? "" : value.getAsString();
    }
}
