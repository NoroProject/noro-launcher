package dev.noro.client.staff.ui;

import dev.noro.client.NoroCore;
import dev.noro.client.staff.CaseIntents;
import dev.noro.client.staff.CaseModels;
import dev.noro.client.staff.NoroStaff;
import dev.noro.client.ui.Duration;
import dev.noro.client.ui.NoroButton;
import dev.noro.client.ui.NoroScreen;
import dev.noro.client.ui.Theme;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Наказание: за какое правило, что за него полагается, на сколько и почему.
 *
 * <p>Порядок шагов тот же, что на сайте, и не случайно: модератор ходит между
 * ними в течение одной смены, а две разные формы для одного действия — это два
 * набора привычек и две возможности ошибиться.
 *
 * <p>В две колонки, потому что экран игры не прокручивается: на сайте та же
 * форма — одна колонка в модальном окне, которое листается, а здесь всё должно
 * поместиться разом. Слева выбор правила, справа само наказание. В один столбец
 * список правил и форма отнимали друг у друга высоту и налезали друг на друга.
 *
 * <p>Рамки срока держит мастер. Здесь они видны только чтобы не отправлять
 * заведомо отказное: решает всё равно он.
 */
public final class PunishScreen extends NoroScreen {

    private static final int LABEL = 3 * Theme.GRID;
    private static final int ROW = 5 * Theme.GRID;
    private static final int GAP = 2 * Theme.GRID;

    private final String caseId;
    private final PunishForm form = new PunishForm();
    private final RulePicker picker = new RulePicker();
    private final PunishFields fields = new PunishFields(form, this::rebuild);

    public PunishScreen(String caseId, String ruleCode) {
        super(Component.translatable("noro.cases.punish.title"));
        this.caseId = caseId;
        form.preselect(ruleCode);
    }

    @Override
    protected Component subtitle() {
        CaseModels.View view = NoroStaff.state().open();
        return view == null || view.brief() == null
                ? null
                : Component.literal(view.brief().target_name());
    }

    /** Ширина правой колонки: на узком окне — половина, дальше не растёт. */
    private int rightWidth() {
        return Math.min(40 * Theme.GRID, contentW / 2);
    }

    @Override
    protected int footerHeight() {
        return NoroButton.HEIGHT + 2 * Theme.GRID;
    }

    @Override
    protected void layout() {
        NoroCore.rules().request();
        int right = rightWidth();
        int left = contentW - right - 4 * Theme.GRID;

        addRenderableWidget(picker.search(font, contentX, contentY + LABEL, left));
        int listTop = contentY + LABEL + ROW + GAP;
        int bottom = footerY;
        picker.bounds(contentX, listTop, left, bottom - GAP - listTop);

        // Правило могло приехать позже, чем открылся экран: спрашиваем до того,
        // как колонка решит, чипы вилок ей показывать или быстрые сроки.
        form.rule();
        fields.layout(this::addRenderableWidget, contentX + contentW - right, contentY, right, bottom - GAP);
        buttons(left, right, bottom);
    }

    /**
     * Главная кнопка — во всю ширину своей колонки, как на сайте.
     *
     * <p>Рядом с «Отменой» она не помещалась: «Предупреждение» и «Отмена» вместе
     * шире колонки, и они налезали друг на друга. Отмена ушла под список правил,
     * где всё равно оставалось пустое место, — и заодно перестала стоять вплотную
     * к необратимому действию.
     */
    private void buttons(int left, int right, int y) {
        Component apply = form.applyLabel();
        addRenderableWidget(NoroButton.of(contentX + contentW - right, y, right, apply,
                        NoroButton.Variant.WARNING, this::submit)
                .needs(NoroStaff.state().can(NoroStaff.PERM_RESOLVE)));

        Component cancel = Component.translatable("gui.cancel");
        addRenderableWidget(
                NoroButton.ghost(contentX, y, NoroButton.widthFor(cancel), cancel, this::onClose));
    }

    /** Пересобрать экран, сохранив набранное руками. */
    private void rebuild() {
        String text = fields.reason();
        clearWidgets();
        layout();
        // Причину форма подставляет из правила сама; набранное руками важнее.
        if (!text.isBlank() && !text.equals(form.reason())) {
            fields.reason(text);
        }
    }

    @Override
    protected void content(GuiGraphics g, int x, int y, int w, int h, int mouseX, int mouseY) {
        Theme.label(g, Component.translatable("noro.cases.punish.rule"), x, y);
        picker.render(g, mouseX, mouseY);
        fields.labels(g);
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (picker.click(mouseX, mouseY)) {
            form.pick(picker.selected());
            rebuild();
            return true;
        }
        return super.mouseClicked(mouseX, mouseY, button);
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double dx, double dy) {
        picker.scroll(dy);
        return true;
    }

    private void submit() {
        long minutes = Duration.parse(fields.duration());
        String text = fields.reason().trim();
        if (Duration.broken(minutes) || text.length() < 3) {
            return;
        }
        NoroStaff.cases()
                .send(new CaseIntents.Punish(caseId, form.kind(), text, form.ruleCode(),
                        Duration.forever(minutes) ? null : minutes * 60));
        onClose();
    }

    @Override
    public void onClose() {
        assert minecraft != null;
        minecraft.setScreen(new CaseScreen(caseId));
    }
}
