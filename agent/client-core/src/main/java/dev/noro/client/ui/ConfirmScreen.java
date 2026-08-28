package dev.noro.client.ui;

import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

/** "Are you sure?" over the calling screen, for things that are hard to undo. */
public final class ConfirmScreen extends NoroScreen {

    private final Screen parent;
    private final Component question;
    private final Component confirmLabel;
    private final Runnable onConfirm;
    private final boolean dangerous;

    public ConfirmScreen(
            Screen parent,
            Component title,
            Component question,
            Component confirmLabel,
            boolean dangerous,
            Runnable onConfirm) {
        super(title);
        this.parent = parent;
        this.question = question;
        this.confirmLabel = confirmLabel;
        this.dangerous = dangerous;
        this.onConfirm = onConfirm;
    }

    @Override
    protected void init() {
        // Smaller than a normal window: one line of text and two buttons.
        super.init();
        windowW = Math.min(width - 8 * Theme.GRID, 64 * Theme.GRID);
        windowH = 24 * Theme.GRID;
        windowX = (width - windowW) / 2;
        windowY = (height - windowH) / 2;
        clearWidgets();
        layout();
    }

    @Override
    protected void layout() {
        int pad = 4 * Theme.GRID;
        int half = (windowW - 2 * pad - Theme.GRID) / 2;
        int y = windowY + windowH - pad - NoroButton.HEIGHT;

        addRenderableWidget(NoroButton.of(
                windowX + pad,
                y,
                half,
                confirmLabel,
                dangerous ? NoroButton.Variant.DANGER : NoroButton.Variant.PRIMARY,
                () -> {
                    onConfirm.run();
                    back();
                }));
        addRenderableWidget(NoroButton.ghost(
                windowX + pad + half + Theme.GRID,
                y,
                half,
                Component.translatable("gui.cancel"),
                this::back));
    }

    private void back() {
        assert minecraft != null;
        minecraft.setScreen(parent);
    }

    @Override
    protected void content(GuiGraphics g, int x, int y, int w, int h, int mouseX, int mouseY) {
        for (var line : font.split(question, w)) {
            g.drawString(font, line, x, y, Theme.TEXT_SECONDARY);
            y += 10;
        }
    }

    @Override
    public void onClose() {
        back();
    }
}
