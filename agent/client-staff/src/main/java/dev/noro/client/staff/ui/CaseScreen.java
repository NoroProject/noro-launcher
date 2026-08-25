package dev.noro.client.staff.ui;

import dev.noro.client.staff.Actions;
import dev.noro.client.staff.CaseIntents;
import dev.noro.client.staff.CaseModels;
import dev.noro.client.staff.CaseState;
import dev.noro.client.staff.CaseStyle;
import dev.noro.client.staff.NoroStaff;
import dev.noro.client.ui.Chip;
import dev.noro.client.ui.NoroButton;
import dev.noro.client.ui.NoroScreen;
import dev.noro.client.ui.RowList;
import dev.noro.client.ui.Tabs;
import dev.noro.client.ui.Theme;
import java.util.List;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Карточка разбора: лента, срез чата и инвентарь — по одной панели за раз.
 *
 * <p>Вкладками, а не колонками: окно узкое, а и лента, и срез — это текст.
 * Две колонки резали обе, а инвентарь всё равно приезжал третьей сущностью и
 * ложился поверх чата. Дела к тому же разного вида: чату нужен срез, грифу —
 * инвентарь, и всё сразу почти никогда не нужно.
 *
 * <p>Открытый экран в мультиплеере значит «я не двигаюсь», и это не то
 * состояние, в котором ловят читера, — поэтому карточка открывается отдельной
 * клавишей. Наблюдение живёт в оверлее.
 */
public final class CaseScreen extends NoroScreen {

    private final CaseState state = NoroStaff.state();
    private final String caseId;

    /** Низ панели действий: от него отсчитываются вкладки. */
    private int actionsBottom;

    private final Tabs tabs = new Tabs(List.of(
            Component.translatable("noro.cases.card.timeline"),
            Component.translatable("noro.cases.card.chat"),
            Component.translatable("noro.cases.card.inventory")));

    public CaseScreen(String caseId) {
        super(Component.translatable("noro.cases.card.title"));
        this.caseId = caseId;
    }

    @Override
    protected Component subtitle() {
        CaseModels.Brief b = brief();
        if (b == null) {
            return null;
        }
        String target = b.target_name() == null ? "—" : b.target_name();
        return Component.literal(b.label() + " · " + target);
    }

    private CaseModels.Brief brief() {
        CaseModels.View view = state.open();
        return view == null ? null : view.brief();
    }

    /** Строка состояния, действия в мире, вкладки и восемь строк под них. */
    @Override
    protected int contentHeight() {
        return 9 * Theme.GRID + 2 * Chip.HEIGHT + Theme.GRID
                + 3 * Theme.GRID + Tabs.HEIGHT + 2 * Theme.GRID + 8 * RowList.ROW;
    }

    @Override
    protected int footerHeight() {
        return NoroButton.HEIGHT + 2 * Theme.GRID;
    }

    @Override
    protected void layout() {
        // Агент должен знать, какое дело на экране: команды в мир номера не
        // несут, а замков у модератора может быть несколько.
        Actions.use(caseId);
        String rule = brief() == null ? null : brief().rule_code();

        actionsBottom = CaseActions.layout(this::addRenderableWidget, caseId,
                contentX, contentY + 9 * Theme.GRID, contentW);

        new CaseFooter(caseId, this::addRenderableWidget, this::open)
                .layout(contentX, footerY, contentW, rule, tabs.active());
    }

    private void open(net.minecraft.client.gui.screens.Screen screen) {
        assert minecraft != null;
        minecraft.setScreen(screen);
    }

    @Override
    protected void content(GuiGraphics g, int x, int y, int w, int h, int mouseX, int mouseY) {
        CaseModels.View view = state.open();
        if (view == null || view.brief() == null) {
            g.drawCenteredString(font, Component.translatable("noro.cases.card.loading"),
                    x + w / 2, y + h / 2, Theme.TEXT_MUTED);
            return;
        }
        status(g, view.brief(), x, y, w);
        Theme.label(g, Component.translatable("noro.cases.action.world"), x, y + 5 * Theme.GRID);

        int top = actionsBottom + 3 * Theme.GRID;
        tabs.render(g, x, top, w, mouseX, mouseY);

        int panel = top + Tabs.HEIGHT + 2 * Theme.GRID;
        int bottom = y + h - 7 * Theme.GRID;
        switch (tabs.active()) {
            case 0 -> Timeline.render(g, font, view.events(), x, panel, w, bottom);
            case 1 -> ChatSlice.render(g, font, view, x, panel, w, bottom);
            default -> Inventory.render(g, font, state.inventory(caseId), x, panel, w, bottom);
        }
        Rejection.render(g, font, state, width, height);
    }

    /** Строка состояния: вес жалоб, пункт свода и статус — темп разбора. */
    private void status(GuiGraphics g, CaseModels.Brief b, int x, int y, int w) {
        g.drawString(font, CaseStyle.reports(b.reports_count(), b.reporters_count()),
                x, y, Theme.TEXT_MUTED);
        if (b.rule_code() != null) {
            g.drawString(font, b.rule_code(), x + 30 * Theme.GRID, y, Theme.ACCENT);
        }
        Component status = Component.translatable("noro.cases.card.status." + b.status());
        g.drawString(font, status, x + w - font.width(status), y,
                CaseStyle.statusColor(b.status()));
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (tabs.click(mouseX, mouseY)) {
            // Кнопка запроса принадлежит вкладке: со сменой вкладки её надо
            // пересобрать, иначе останется чужая.
            clearWidgets();
            layout();
            return true;
        }
        // Клик по строке среза выбирает цитату — она станет причиной наказания.
        if (button == 0 && tabs.active() == 1 && ChatSlice.click(state.open(), mouseX, mouseY)) {
            return true;
        }
        return super.mouseClicked(mouseX, mouseY, button);
    }

    /**
     * Закрытие карточки не бросает дело.
     *
     * <p>Раньше Esc слал {@code CloseCase}, и лаунчер переставал присылать
     * обновления: панель, открытая заново, показывала очередь, а HUD в углу
     * гас — хотя дело так и оставалось за модератором. Дело отпускают кнопкой
     * «вернуть в очередь» или закрывают вердиктом, а не выходом из окна.
     */
    @Override
    public void onClose() {
        super.onClose();
    }
}
