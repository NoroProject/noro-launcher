package dev.noro.client.staff.ui;

import dev.noro.client.staff.CaseIntents;
import dev.noro.client.staff.NoroStaff;
import dev.noro.client.ui.NoroButton;
import dev.noro.client.ui.NoroScreen;
import dev.noro.client.ui.Select;
import dev.noro.client.ui.TextField;
import dev.noro.client.ui.Theme;
import java.util.List;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Закрытие дела вердиктом.
 *
 * <p>Текст резолюции увидит автор жалобы — без него человек не узнаёт, что его
 * обращение вообще прочитали, и через месяц перестаёт писать вовсе.
 */
public final class ResolveScreen extends NoroScreen {

    private static final List<String> VERDICTS = List.of("confirmed", "rejected", "insufficient");

    private final String caseId;
    private final String ruleCode;
    private TextField resolution;
    private Select<String> verdict;

    public ResolveScreen(String caseId, String ruleCode) {
        super(Component.translatable("noro.cases.resolve.title"));
        this.caseId = caseId;
        this.ruleCode = ruleCode;
    }

    @Override
    protected Component subtitle() {
        return Component.translatable("noro.cases.resolve.hint");
    }

    @Override
    protected void layout() {
        int pad = 4 * Theme.GRID;
        int x = windowX + pad;
        int w = windowW - 2 * pad;
        int y = windowY + 16 * Theme.GRID;
        int half = (w - Theme.GRID) / 2;

        resolution = TextField.of(
                font,
                x,
                y,
                w,
                Component.translatable("noro.cases.resolve.hint"),
                Component.translatable("noro.cases.resolve.hint"));
        addRenderableWidget(resolution);

        verdict = Select.of(
                x, y + 7 * Theme.GRID, w, VERDICTS, ResolveScreen::verdictLabel, v -> {});
        addRenderableWidget(verdict);
        addRenderableWidget(NoroButton.primary(
                x,
                y + 14 * Theme.GRID,
                half,
                Component.translatable("noro.cases.resolve.close"),
                this::submit));
        addRenderableWidget(NoroButton.ghost(
                x + half + Theme.GRID,
                y + 14 * Theme.GRID,
                half,
                Component.translatable("gui.cancel"),
                this::onClose));
    }

    private static Component verdictLabel(String verdict) {
        return Component.translatable("noro.cases.verdict." + verdict);
    }

    private void submit() {
        NoroStaff.cases()
                .send(new CaseIntents.Resolve(
                        caseId, verdict.value(), resolution.getValue().trim(), ruleCode));
        assert minecraft != null;
        // Дело закрыто — возвращаемся в очередь, а не в пустую карточку.
        minecraft.setScreen(new QueueScreen());
    }

    @Override
    protected void content(GuiGraphics g, int x, int y, int w, int h, int mouseX, int mouseY) {
        if (ruleCode != null) {
            g.drawString(font, ruleCode, x, y, Theme.ACCENT);
        }
    }

    @Override
    public void onClose() {
        assert minecraft != null;
        minecraft.setScreen(new CaseScreen(caseId));
    }
}
