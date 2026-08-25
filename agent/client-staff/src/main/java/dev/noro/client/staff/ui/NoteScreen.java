package dev.noro.client.staff.ui;

import dev.noro.client.staff.CaseIntents;
import dev.noro.client.staff.NoroStaff;
import dev.noro.client.ui.NoroButton;
import dev.noro.client.ui.NoroScreen;
import dev.noro.client.ui.Theme;
import net.minecraft.client.gui.GuiGraphics;
import dev.noro.client.ui.TextArea;
import net.minecraft.network.chat.Component;

/**
 * Заметка к делу.
 *
 * <p>Многострочно, а не одной командой: «что вы увидели, своими словами» не
 * помещается в чат-строку, а через месяц апелляции это единственное, по чему
 * разбор вообще читается.
 */
public final class NoteScreen extends NoroScreen {

    private final String caseId;
    private TextArea text;

    public NoteScreen(String caseId) {
        super(Component.translatable("noro.cases.note.title"));
        this.caseId = caseId;
    }

    @Override
    protected Component subtitle() {
        return Component.translatable("noro.cases.note.hint");
    }

    @Override
    protected void layout() {
        int pad = 4 * Theme.GRID;
        int x = windowX + pad;
        int w = windowW - 2 * pad;
        int y = windowY + 14 * Theme.GRID;
        int h = windowH - 26 * Theme.GRID;
        int half = (w - Theme.GRID) / 2;

        text = TextArea.of(
                font,
                x,
                y,
                w,
                h,
                Component.translatable("noro.cases.note.hint"),
                Component.translatable("noro.cases.note.title"));
        addRenderableWidget(text);

        addRenderableWidget(NoroButton.primary(
                x,
                y + h + Theme.GRID,
                half,
                Component.translatable("noro.cases.note.add"),
                this::submit));
        addRenderableWidget(NoroButton.ghost(
                x + half + Theme.GRID,
                y + h + Theme.GRID,
                half,
                Component.translatable("gui.cancel"),
                this::onClose));
    }

    private void submit() {
        String body = text.getValue().trim();
        if (!body.isEmpty()) {
            NoroStaff.cases().send(new CaseIntents.AddNote(caseId, body));
        }
        onClose();
    }

    @Override
    protected void content(GuiGraphics g, int x, int y, int w, int h, int mouseX, int mouseY) {
        // Поле занимает всю область: рисовать поверх него нечего.
    }

    @Override
    public void onClose() {
        assert minecraft != null;
        minecraft.setScreen(new CaseScreen(caseId));
    }
}
