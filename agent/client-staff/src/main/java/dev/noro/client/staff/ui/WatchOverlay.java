package dev.noro.client.staff.ui;

import dev.noro.client.staff.CaseModels;
import dev.noro.client.staff.CaseStyle;
import dev.noro.client.staff.CaseWheel;
import dev.noro.client.staff.NoroStaff;
import dev.noro.client.staff.Telemetry;
import dev.noro.client.ui.Hud;
import dev.noro.client.ui.Surface;
import dev.noro.client.ui.Theme;
import java.util.List;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * HUD разбора: цель, состояние, числа слежки и живой чат.
 *
 * <p>Оверлей не отнимает управление. Полноэкранный GUI в мультиплеере значит «я
 * не двигаюсь», и это не то состояние, в котором ловят читера, — поэтому пока
 * идёт наблюдение, панель остаётся плашкой в углу.
 *
 * <p>Высота считается по содержимому, а не задана числом: без слежки и без чата
 * плашка была наполовину пустой, а строки чата не влезали и уходили за край
 * экрана мимо рамки.
 */
public final class WatchOverlay {

    private static final int PAD = 2 * Theme.GRID;
    private static final int LINE = 10;

    /** Ширина плашки: шире — и она начинает спорить с прицелом. */
    private static final int W = 42 * Theme.GRID;

    /** Строк чата: последние четыре — это то, из-за чего обычно и зовут. */
    private static final int CHAT_LINES = 4;

    private WatchOverlay() {}

    public static void render(GuiGraphics g, int width, int height) {
        CaseModels.View view = NoroStaff.state().open();
        if (view == null || view.brief() == null) {
            CaseWheel.wheel().render(g);
            return;
        }
        Font font = Theme.font();
        CaseModels.Brief brief = view.brief();
        List<CaseModels.Message> chat = view.messages();

        int x = width - W - Hud.MARGIN;
        int y = Hud.MARGIN + 6 * Theme.GRID;
        Surface.window(g, x, y, W, height(chat));

        int cursor = header(g, font, brief, x, y);
        cursor = telemetry(g, font, x, cursor);
        chat(g, font, chat, x, cursor);
        CaseWheel.wheel().render(g);
    }

    /** Сколько места займёт плашка: секции есть не всегда. */
    private static int height(List<CaseModels.Message> chat) {
        int h = 11 * Theme.GRID + 1;
        if (Telemetry.watching()) {
            h += Theme.GRID + 3 * LINE + Theme.GRID;
        }
        if (chat != null && !chat.isEmpty()) {
            h += Theme.GRID + Math.min(CHAT_LINES, chat.size()) * LINE + Theme.GRID;
        }
        return h;
    }

    /**
     * Шапка: номер дела, ник и состояние.
     *
     * <p>Полоса слева цветом состояния — то, что видно, не читая: жёлтая значит
     * «дело ещё никто не взял», зелёная — «в работе».
     *
     * @return верх следующей секции
     */
    private static int header(GuiGraphics g, Font font, CaseModels.Brief brief, int x, int y) {
        int h = 11 * Theme.GRID;
        int accent = CaseStyle.statusColor(brief.status());
        g.fill(x + 2, y + 2, x + W - 2, y + h, Theme.BG_CARD);
        g.fill(x + 2, y + 2, x + 2 + Theme.GRID / 2, y + h, accent);
        g.fill(x + 2, y + h, x + W - 2, y + h + 1, Theme.BORDER_STRONG);

        int left = x + PAD + Theme.GRID / 2;
        g.drawString(font, brief.label(), left, y + PAD, Theme.TEXT_MUTED, false);

        Component status = Component.translatable("noro.cases.card.status." + brief.status());
        g.drawString(font, status, x + W - PAD - font.width(status), y + PAD, accent, false);

        String target = brief.target_name() == null ? "—" : brief.target_name();
        g.drawString(font, Theme.clip(Component.literal(target), W - 2 * PAD),
                left, y + PAD + LINE + Theme.GRID / 2, Theme.TEXT, false);
        return y + h + 1;
    }

    /** Числа слежки — их не даст ни сайт, ни глаза. */
    private static int telemetry(GuiGraphics g, Font font, int x, int y) {
        if (!Telemetry.watching()) {
            return y;
        }
        Surface.divider(g, x + 1, y, W - 2);
        Telemetry.render(g, font, x + PAD, y + Theme.GRID);
        return y + Theme.GRID + 3 * LINE + Theme.GRID;
    }

    /** Последние строки цели: за ними и следят, а листать некогда. */
    private static void chat(GuiGraphics g, Font font, List<CaseModels.Message> messages, int x, int y) {
        if (messages == null || messages.isEmpty()) {
            return;
        }
        Surface.divider(g, x + 1, y, W - 2);
        int row = y + Theme.GRID;
        for (int i = Math.max(0, messages.size() - CHAT_LINES); i < messages.size(); i++) {
            CaseModels.Message m = messages.get(i);
            // Ник отдельно от текста: в четырёх строках без пробела глазу не за
            // что зацепиться, а обрезать надо именно текст, а не ник.
            String who = m.sender_name() + ": ";
            int nick = font.width(who);
            g.drawString(font, who, x + PAD, row, Theme.TEXT_SECONDARY, false);
            g.drawString(font, font.plainSubstrByWidth(m.content(), W - 2 * PAD - nick),
                    x + PAD + nick, row, Theme.TEXT_MUTED, false);
            row += LINE;
        }
    }
}
