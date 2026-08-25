package dev.noro.client.ui;

import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.MultiLineEditBox;
import net.minecraft.network.chat.Component;

/**
 * Многострочное поле в стиле лаунчера.
 *
 * <p>Ванильный {@link MultiLineEditBox} рисует себе чёрный прямоугольник с
 * белой рамкой — посреди тёмно-синей панели это дыра. Наследуемся и меняем
 * только фон: перенос строк, прокрутка и выделение остаются ванильными, и
 * переписывать их ради вида было бы дорого и опасно.
 */
public final class TextArea extends MultiLineEditBox {

    private TextArea(Font font, int x, int y, int w, int h, Component placeholder, Component label) {
        super(font, x, y, w, h, placeholder, label);
    }

    public static TextArea of(
            Font font, int x, int y, int w, int h, Component placeholder, Component label) {
        return new TextArea(font, x, y, w, h, placeholder, label);
    }

    @Override
    protected void renderBackground(GuiGraphics g) {
        g.fill(getX(), getY(), getX() + width, getY() + height, Theme.BG_INPUT);
        Surface.outline(g, getX(), getY(), width, height, isFocused() ? Theme.CTA : Theme.BORDER);
    }
}
