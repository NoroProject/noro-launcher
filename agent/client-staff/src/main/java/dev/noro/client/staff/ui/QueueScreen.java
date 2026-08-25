package dev.noro.client.staff.ui;

import dev.noro.client.staff.CaseIntents;
import dev.noro.client.staff.CaseModels;
import dev.noro.client.staff.NoroStaff;
import dev.noro.client.ui.NoroButton;
import dev.noro.client.ui.NoroScreen;
import dev.noro.client.ui.RowList;
import dev.noro.client.ui.TextField;
import dev.noro.client.ui.Theme;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Очередь дел: цель, вес жалоб, кто ведёт.
 *
 * <p>Взять дело — одно действие: Enter на строке ставит замок и открывает
 * карточку. Не «взял → закрыл окно → набрал {@code /case tp}».
 *
 * <p>Ищет и режет на страницы мастер. Раньше очередь приезжала целиком и
 * фильтровалась здесь — то есть поиск находил только то, что уже приехало, а на
 * большом сервере это была первая сотня дел из нескольких сотен.
 */
public final class QueueScreen extends NoroScreen {

    /** Сколько дел в странице. Столько же присылает лаунчер — см. {@code QUEUE_PAGE}. */
    private static final int PAGE = 10;

    private final RowList<CaseModels.Brief> list = new RowList<>(QueueTable::row);

    private TextField search;

    /** Что запрошено сейчас. Экран пересоздаётся, состояние живёт в поле. */
    private String query;

    private long offset;

    public QueueScreen() {
        super(Component.translatable("noro.cases.queue.title"));
    }

    /** «11–20 из 348»: сколько дел всего, а не сколько влезло на экран. */
    @Override
    protected Component subtitle() {
        long total = NoroStaff.state().queueTotal();
        int shown = NoroStaff.state().queue().size();
        if (total <= shown) {
            return Component.translatable("noro.cases.queue.count", total);
        }
        long from = NoroStaff.state().queueOffset() + 1;
        return Component.translatable("noro.cases.queue.range", from, from + shown - 1, total);
    }

    /**
     * Ровно столько, сколько дел в странице.
     *
     * <p>Не меньше четырёх строк: окно, схлопнувшееся до одной, прыгает в
     * размере на каждое взятое дело.
     */
    @Override
    protected int contentHeight() {
        int rows = Math.max(4, NoroStaff.state().queue().size());
        return 10 * Theme.GRID + rows * RowList.ROW;
    }

    @Override
    protected int footerHeight() {
        return NoroButton.HEIGHT + 2 * Theme.GRID;
    }

    @Override
    protected void layout() {
        request();

        search = addRenderableWidget(TextField.of(
                font,
                contentX,
                contentY,
                contentW,
                Component.translatable("noro.cases.queue.search")));
        search.setValue(query == null ? "" : query);
        // Ищем по мере набора: страница приезжает с мастера, и лишний Enter
        // между «набрал ник» и «увидел дело» здесь ничего не экономит.
        search.setResponder(this::onSearch);

        Component take = Component.translatable("noro.cases.queue.take");
        int w = NoroButton.widthFor(take);
        addRenderableWidget(NoroButton.primary(contentX + contentW - w, footerY, w, take, this::take)
                .needs(NoroStaff.state().can(NoroStaff.PERM_CLAIM)));

        Component prev = Component.translatable("noro.cases.queue.prev");
        Component next = Component.translatable("noro.cases.queue.next");
        int pw = NoroButton.widthFor(prev);
        int nw = NoroButton.widthFor(next);
        addRenderableWidget(NoroButton.ghost(contentX, footerY, pw, prev, () -> page(-1))
                .needs(offset > 0));
        addRenderableWidget(
                NoroButton.ghost(contentX + pw + Theme.GRID, footerY, nw, next, () -> page(1))
                        .needs(offset + PAGE < NoroStaff.state().queueTotal()));
    }

    /** Новый запрос — всегда с первой страницы: иначе он начинается с середины. */
    private void onSearch(String value) {
        String fresh = value == null || value.isBlank() ? null : value.trim();
        if (java.util.Objects.equals(fresh, query)) {
            return;
        }
        query = fresh;
        offset = 0;
        request();
    }

    private void page(int step) {
        long moved = offset + (long) step * PAGE;
        long total = NoroStaff.state().queueTotal();
        offset = Math.max(0, Math.min(moved, Math.max(0, total - 1)));
        request();
    }

    private void request() {
        NoroStaff.cases().send(new CaseIntents.RequestQueuePage(query, offset));
    }

    @Override
    protected void content(GuiGraphics g, int x, int y, int w, int h, int mouseX, int mouseY) {
        list.items(NoroStaff.state().queue());
        int top = y + 10 * Theme.GRID;
        if (list.empty()) {
            String key = query == null ? "noro.cases.queue.empty" : "noro.cases.queue.nothing-found";
            g.drawCenteredString(
                    font, Component.translatable(key), x + w / 2, y + h / 2, Theme.TEXT_MUTED);
            return;
        }

        QueueTable.header(g, font, x, y + 7 * Theme.GRID, w);
        list.render(g, x, top, w, y + h - top, mouseX, mouseY);
    }

    /** Enter — замок и карточка разом: одно действие вместо трёх. */
    private void take() {
        CaseModels.Brief brief = list.selected();
        if (brief == null) {
            return;
        }
        if (brief.claimed_by() == null && NoroStaff.state().can(NoroStaff.PERM_CLAIM)) {
            NoroStaff.cases().send(new CaseIntents.Claim(brief.id()));
        }
        NoroStaff.cases().send(new CaseIntents.OpenCase(brief.id()));
        // Агент должен считать открытым то же дело, что и панель.
        dev.noro.client.staff.Actions.use(brief.id());
        assert minecraft != null;
        minecraft.setScreen(new CaseScreen(brief.id()));
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        return list.click(mouseX, mouseY) || super.mouseClicked(mouseX, mouseY, button);
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double dx, double dy) {
        list.scroll(dy);
        return true;
    }

    @Override
    public boolean keyPressed(int key, int scan, int modifiers) {
        // Пока курсор в поиске, стрелки и Enter принадлежат ему: иначе набрать
        // ник нельзя — первая же стрелка уводит выделение в списке.
        if (search != null && search.isFocused() && key != 257 && key != 335) {
            return super.keyPressed(key, scan, modifiers);
        }
        switch (key) {
            case 264 -> list.move(1);
            case 265 -> list.move(-1);
            case 257, 335 -> take();
            default -> {
                return super.keyPressed(key, scan, modifiers);
            }
        }
        return true;
    }
}
