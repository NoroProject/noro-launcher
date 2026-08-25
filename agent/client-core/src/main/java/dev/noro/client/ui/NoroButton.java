package dev.noro.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.network.chat.Component;

/**
 * Кнопка лаунчера: прямоугольник без ванильной текстуры.
 *
 * <p>Варианты те же, что у {@code AtomButton} на сайте, и значат то же самое.
 * Главное действие на экране одно и оно кремовое — как «Играть» в лаунчере;
 * остальные приглушены. Это не украшение: когда все кнопки выглядят одинаково,
 * нужную ищут глазами каждый раз заново.
 */
public final class NoroButton extends AbstractButton {

    /** Высота: пять шагов сетки — минимум, в который влезает текст с воздухом. */
    public static final int HEIGHT = 5 * Theme.GRID;

    public enum Variant {
        /** Единственное на экране действие, ради которого его открыли. */
        PRIMARY,
        /** Обычное действие: их на экране может быть сколько угодно. */
        SECONDARY,
        /** Без заливки — для мест, где рамка уже есть у соседа. */
        GHOST,
        /** Действие, которое трудно отменить. */
        DANGER,
        /** Действие с последствиями, но обратимое. */
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

    /** Отступ от текста до края кнопки с каждой стороны. */
    private static final int PAD = 3 * Theme.GRID;

    public static NoroButton of(
            int x, int y, int w, Component label, Variant variant, Runnable onPress) {
        return new NoroButton(x, y, w, label, variant, onPress);
    }

    /**
     * Ширина по подписи, а не наугад.
     *
     * <p>Кнопка с заданной шириной и длинным текстом — самый заметный способ
     * сломать экран: подпись вылезает за края и накрывает соседей. Здесь ширину
     * задаёт текст, а сетку держит округление вверх до её шага.
     */
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
     * Погасить кнопку, если права нет.
     *
     * <p>Это удобство, а не защита: решает мастер, и его отказ — единственная
     * проверка, которой стоит верить. Но прятать кнопку значило бы врать про
     * то, чего у человека нет: он не узнает, что действие вообще существует.
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
        // Блик сверху и тень снизу: одна заливка выглядит нарисованной на стене,
        // а не лежащей на ней. Полпикселя разницы хватает — кнопка сама по себе
        // маленькая, и большего объёма ей не нужно.
        g.fill(getX(), getY(), getX() + width, getY() + 1, Theme.lift(fill, 0x18));
        g.fill(getX(), getY() + height - 1, getX() + width, getY() + height, Theme.SHADOW);
        if (variant != Variant.PRIMARY) {
            border(g, hovered);
        }
        // Подпись обрезается по ширине кнопки: даже если её задали вручную и
        // текст не влез, он не должен вылезать на соседние виджеты.
        var font = Minecraft.getInstance().font;
        String label = font.plainSubstrByWidth(getMessage().getString(), width - 2 * PAD);
        // Без тени: на кремовой заливке тёмная тень под тёмным текстом делает
        // из подписи грязь. Ванильный drawCenteredString рисует её всегда,
        // поэтому центрируем сами.
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
