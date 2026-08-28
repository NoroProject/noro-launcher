package dev.noro.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.network.chat.Component;

/**
 * A small toggle, like {@code noro-chip} on the site. Differs from a button in
 * meaning rather than size: a chip shows state, so the selected one stays lit.
 */
public final class Chip extends AbstractButton {

    public static final int HEIGHT = 5 * Theme.GRID;

    private static final int PAD = 2 * Theme.GRID;

    private final Runnable onPress;
    private boolean on;

    private Chip(int x, int y, int w, Component label, boolean on, Runnable onPress) {
        super(x, y, w, HEIGHT, label);
        this.on = on;
        this.onPress = onPress;
    }

    public static int widthFor(Component label) {
        return Minecraft.getInstance().font.width(label) + 2 * PAD;
    }

    public static Chip of(int x, int y, Component label, boolean on, Runnable onPress) {
        return new Chip(x, y, widthFor(label), label, on, onPress);
    }

    public void on(boolean value) {
        this.on = value;
    }

    @Override
    public void onPress() {
        onPress.run();
    }

    @Override
    protected void renderWidget(GuiGraphics g, int mouseX, int mouseY, float partial) {
        int fill = on ? Theme.BG_CARD_HOVER : Theme.BG_INPUT;
        g.fill(getX(), getY(), getX() + width, getY() + height, fill);
        g.fill(getX(), getY(), getX() + width, getY() + 1, Theme.lift(fill, 0x14));
        Surface.outline(
                g,
                getX(),
                getY(),
                width,
                height,
                on ? Theme.CTA : isHovered() ? Theme.BORDER_STRONG : Theme.BORDER);
        g.drawCenteredString(
                Minecraft.getInstance().font,
                getMessage(),
                getX() + width / 2,
                getY() + (height - 8) / 2,
                on ? Theme.TEXT : Theme.TEXT_SECONDARY);
    }

    @Override
    protected void updateWidgetNarration(NarrationElementOutput output) {
        defaultButtonNarrationText(output);
    }
}
