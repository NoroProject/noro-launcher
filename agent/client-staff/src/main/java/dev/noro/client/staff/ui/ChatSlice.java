package dev.noro.client.staff.ui;

import dev.noro.client.staff.CaseModels;
import dev.noro.client.ui.Theme;
import java.util.List;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Срез чата вокруг события.
 *
 * <p>Дела про чат — самые частые, и телепорт для них не нужен вовсе: открыл
 * дело, увидел срез, щёлкнул сообщение, выдал мут. Отсюда срез на пол-экрана, а
 * не свёрнутой панелью.
 */
final class ChatSlice {

    private static final int ROW = 10;

    /** Сообщение, выбранное для цитаты. Индекс, а не ссылка: срез перечитывается. */
    static int selected = -1;

    /** Где нарисован срез — чтобы клик попал в ту же строку, что видит глаз. */
    private static int lastX;
    private static int lastTop;
    private static int lastFrom;
    private static int lastRows;

    private ChatSlice() {}

    /**
     * Клик по строке среза. Выбранное сообщение подставится в причину
     * наказания: копировать руками ничего не нужно.
     */
    static boolean click(CaseModels.View view, double mouseX, double mouseY) {
        if (view == null || !view.chat_allowed() || view.messages() == null) {
            return false;
        }
        if (mouseX < lastX - Theme.GRID || mouseY < lastTop) {
            return false;
        }
        int row = (int) ((mouseY - lastTop) / ROW);
        if (row < 0 || row >= lastRows) {
            return false;
        }
        int index = lastFrom + row;
        if (index >= view.messages().size()) {
            return false;
        }
        selected = selected == index ? -1 : index;
        return true;
    }

    static void render(
            GuiGraphics g, Font font, CaseModels.View view, int x, int y, int w, int bottom) {
        // Заголовка здесь нет: его уже несёт вкладка, под которой рисуют срез, и
        // вторая такая же строка читалась как «чат вокруг события: чат вокруг
        // события».
        if (!view.chat_allowed()) {
            g.drawString(
                    font,
                    Component.translatable("noro.cases.card.chat.forbidden"),
                    x,
                    y,
                    Theme.ERROR);
            return;
        }
        List<CaseModels.Message> messages = view.messages();
        if (messages == null || messages.isEmpty()) {
            g.drawString(
                    font,
                    Component.translatable("noro.cases.card.chat.empty"),
                    x,
                    y,
                    Theme.TEXT_MUTED);
            return;
        }

        int top = y;
        int rows = (bottom - top) / ROW;
        int from = Math.max(0, messages.size() - rows);
        lastX = x;
        lastTop = top;
        lastFrom = from;
        lastRows = messages.size() - from;
        for (int i = from; i < messages.size(); i++) {
            row(g, font, messages.get(i), x, top + (i - from) * ROW, w, i == selected);
        }
    }

    private static void row(
            GuiGraphics g, Font font, CaseModels.Message m, int x, int y, int w, boolean active) {
        if (active) {
            g.fill(x - Theme.GRID, y - 1, x + w, y + ROW - 2, Theme.BG_CARD_HOVER);
        }
        g.drawString(font, Timeline.clock(m.at()), x, y, Theme.TEXT_MUTED);
        g.drawString(font, m.sender_name(), x + 9 * Theme.GRID, y, channelColor(m.channel()));
        g.drawString(
                font,
                font.plainSubstrByWidth(m.content(), w - 30 * Theme.GRID),
                x + 30 * Theme.GRID,
                y,
                Theme.TEXT);
    }

    /** Личку и команды видно отдельно: за ними разные права и разный вес. */
    private static int channelColor(String channel) {
        if (channel == null) {
            return Theme.TEXT_MUTED;
        }
        return switch (channel) {
            case "private" -> Theme.ACCENT;
            case "command" -> Theme.ERROR;
            default -> Theme.TEXT_MUTED;
        };
    }
}
