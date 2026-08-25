package dev.noro.client.ui;

import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.network.chat.Component;

/**
 * Поле ввода в стиле лаунчера.
 *
 * <p>Наследует ванильный {@link EditBox} и переписывает только фон: каретка,
 * выделение, буфер обмена и ввод с раскладок — работа, которую переписывать
 * незачем и опасно. Меняется ровно то, что видно.
 */
public final class TextField extends EditBox {

    public static final int HEIGHT = 5 * Theme.GRID;

    /** Подсказка внутри пустого поля — как placeholder на сайте. */
    private final Component placeholder;

    private TextField(Font font, int x, int y, int w, Component label, Component placeholder) {
        super(font, x, y, w, HEIGHT, label);
        this.placeholder = placeholder;
        setBordered(false);
        setMaxLength(256);
        setTextColor(Theme.TEXT);
        setTextColorUneditable(Theme.TEXT_MUTED);
    }

    public static TextField of(Font font, int x, int y, int w, Component label) {
        return new TextField(font, x, y, w, label, null);
    }

    public static TextField of(
            Font font, int x, int y, int w, Component label, Component placeholder) {
        return new TextField(font, x, y, w, label, placeholder);
    }

    @Override
    public void renderWidget(GuiGraphics g, int mouseX, int mouseY, float partial) {
        g.fill(getX(), getY(), getX() + width, getY() + height, Theme.BG_INPUT);
        // Фокус подсвечивается рамкой, а не свечением: в тёмной теме свечение
        // читается как ошибка.
        Surface.outline(
                g,
                getX(),
                getY(),
                width,
                height,
                isFocused() ? Theme.CTA : Theme.BORDER);

        // Ванильный EditBox рисует текст от своего левого края; сдвигаем его
        // внутрь рамки, иначе первая буква прилипает к линии.
        setX(getX() + 2 * Theme.GRID);
        setY(getY() + (height - 8) / 2);
        super.renderWidget(g, mouseX, mouseY, partial);
        setX(getX() - 2 * Theme.GRID);
        setY(getY() - (height - 8) / 2);

        if (placeholder != null && getValue().isEmpty() && !isFocused()) {
            g.drawString(
                    net.minecraft.client.Minecraft.getInstance().font,
                    placeholder,
                    getX() + 2 * Theme.GRID,
                    getY() + (height - 8) / 2,
                    Theme.TEXT_MUTED,
                    false);
        }
    }
}
