package dev.noro.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.network.chat.Component;

/**
 * Launcher-styled button: a plain rectangle, no vanilla texture. Variants match
 * {@code AtomButton} on the site and mean the same thing — at most one PRIMARY
 * per screen.
 */
public final class NoroButton extends AbstractButton {

    public static final int HEIGHT = 5 * Theme.GRID;

    public enum Variant {
        /** The one action the screen was opened for. */
        PRIMARY,
        SECONDARY,
        /** No fill, for places where a neighbour already draws the border. */
        GHOST,
        /** Hard to undo. */
        DANGER,
        /** Has consequences, but reversible. */
        WARNING
    }

    private final Runnable onPress;
    private final Variant variant;
    private boolean allowed = true;

    private NoroButton(int x, int y, int w, Component label, Variant variant, Runnable onPress) {
        super(x, y, w, HEIGHT, label);
        this.variant = variant;
        this.onPress = onPress;
    }

    private static final int PAD = 3 * Theme.GRID;

    public static NoroButton of(
            int x, int y, int w, Component label, Variant variant, Runnable onPress) {
        return new NoroButton(x, y, w, label, variant, onPress);
    }

    /** Width from the label, rounded up to the grid step. */
    public static int widthFor(Component label) {
        int text = Minecraft.getInstance().font.width(label) + 2 * PAD;
        return (text + Theme.GRID - 1) / Theme.GRID * Theme.GRID;
    }

    public static NoroButton fit(int x, int y, Component label, Variant variant, Runnable onPress) {
        return new NoroButton(x, y, widthFor(label), label, variant, onPress);
    }

    public static NoroButton primary(int x, int y, int w, Component label, Runnable onPress) {
        return of(x, y, w, label, Variant.PRIMARY, onPress);
    }

    public static NoroButton secondary(int x, int y, int w, Component label, Runnable onPress) {
        return of(x, y, w, label, Variant.SECONDARY, onPress);
    }

    public static NoroButton ghost(int x, int y, int w, Component label, Runnable onPress) {
        return of(x, y, w, label, Variant.GHOST, onPress);
    }

    public static NoroButton danger(int x, int y, int w, Component label, Runnable onPress) {
        return of(x, y, w, label, Variant.DANGER, onPress);
    }

    /**
     * Grey the button out when the permission is missing. Convenience, not
     * enforcement — the master's 403 is the only check that counts. Greyed rather
     * than hidden, so people can see the action exists.
     */
    public NoroButton needs(boolean permitted) {
        this.allowed = permitted;
        this.active = permitted;
        return this;
    }

    @Override
    public void onPress() {
        if (allowed) {
            onPress.run();
        }
    }

    @Override
    protected void renderWidget(GuiGraphics g, int mouseX, int mouseY, float partial) {
        boolean hovered = isHovered() && allowed;
        int fill = fill(hovered);
        int text = text();
        g.fill(getX(), getY(), getX() + width, getY() + height, fill);
        // Highlight on top, shadow underneath — a flat fill reads as painted on
        // the wall rather than lying on it.
        g.fill(getX(), getY(), getX() + width, getY() + 1, Theme.lift(fill, 0x18));
        g.fill(getX(), getY() + height - 1, getX() + width, getY() + height, Theme.SHADOW);
        if (variant != Variant.PRIMARY) {
            border(g, hovered);
        }
        // Clip the label: width can be set by hand, and an overlong caption must
        // not spill onto neighbouring widgets.
        var font = Minecraft.getInstance().font;
        String label = font.plainSubstrByWidth(getMessage().getString(), width - 2 * PAD);
        // Centred by hand because drawCenteredString always draws a text shadow,
        // and a dark shadow under dark text on the cream fill turns to mud.
        g.drawString(
                font,
                label,
                getX() + (width - font.width(label)) / 2,
                getY() + (height - 8) / 2,
                text,
                variant != Variant.PRIMARY);
    }

    private int fill(boolean hovered) {
        if (!allowed) {
            return Theme.BG_INPUT;
        }
        return switch (variant) {
            case PRIMARY -> hovered ? Theme.CTA_HOVER : Theme.CTA;
            case SECONDARY -> hovered ? Theme.BG_CARD_HOVER : Theme.BG_CARD;
            case GHOST -> hovered ? Theme.BG_CARD : 0x00000000;
            case DANGER -> hovered ? Theme.BG_CARD_HOVER : Theme.BG_CARD;
            case WARNING -> hovered ? Theme.BG_CARD_HOVER : Theme.BG_CARD;
        };
    }

    private int text() {
        if (!allowed) {
            return Theme.TEXT_MUTED;
        }
        return switch (variant) {
            case PRIMARY -> Theme.ON_CTA;
            case DANGER -> Theme.ERROR;
            case WARNING -> Theme.WARNING;
            default -> Theme.TEXT;
        };
    }

    private void border(GuiGraphics g, boolean hovered) {
        int color = switch (variant) {
            case DANGER -> Theme.ERROR;
            case WARNING -> Theme.WARNING;
            default -> hovered ? Theme.BORDER_STRONG : Theme.BORDER;
        };
        Surface.outline(g, getX(), getY(), width, height, color);
    }

    @Override
    protected void updateWidgetNarration(NarrationElementOutput output) {
        defaultButtonNarrationText(output);
    }
}
