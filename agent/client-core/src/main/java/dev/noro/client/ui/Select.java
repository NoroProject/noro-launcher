package dev.noro.client.ui;

import java.util.List;
import java.util.function.Consumer;
import java.util.function.Function;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.network.chat.Component;

/**
 * Выбор из нескольких значений.
 *
 * <p>Колесом, а не выпадающим списком: вариантов в наших полях три-четыре — вид
 * наказания, срок, вердикт, — и раскрывающийся список на такую горсть только
 * добавляет клик. Левый клик вперёд, правый назад, колесо мыши тоже.
 *
 * @param <T> что выбираем
 */
public final class Select<T> extends AbstractButton {

    public static final int HEIGHT = 5 * Theme.GRID;

    private final List<T> options;
    private final Function<T, Component> label;
    private final Consumer<T> onChange;
    private int index;

    private Select(
            int x,
            int y,
            int w,
            List<T> options,
            Function<T, Component> label,
            Consumer<T> onChange) {
        super(x, y, w, HEIGHT, Component.empty());
        this.options = options;
        this.label = label;
        this.onChange = onChange;
    }

    public static <T> Select<T> of(
            int x,
            int y,
            int w,
            List<T> options,
            Function<T, Component> label,
            Consumer<T> onChange) {
        return new Select<>(x, y, w, options, label, onChange);
    }

    public T value() {
        return options.isEmpty() ? null : options.get(index);
    }

    /** Выставить значение снаружи — например, восстановив прошлый выбор. */
    public Select<T> value(T value) {
        int at = options.indexOf(value);
        if (at >= 0) {
            index = at;
        }
        return this;
    }

    private void step(int delta) {
        if (options.isEmpty()) {
            return;
        }
        index = Math.floorMod(index + delta, options.size());
        onChange.accept(value());
    }

    @Override
    public void onPress() {
        step(1);
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (button == 1 && isMouseOver(mouseX, mouseY)) {
            step(-1);
            return true;
        }
        return super.mouseClicked(mouseX, mouseY, button);
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double dx, double dy) {
        if (isMouseOver(mouseX, mouseY)) {
            step(dy > 0 ? -1 : 1);
            return true;
        }
        return false;
    }

    @Override
    protected void renderWidget(GuiGraphics g, int mouseX, int mouseY, float partial) {
        boolean hovered = isHovered();
        g.fill(
                getX(),
                getY(),
                getX() + width,
                getY() + height,
                hovered ? Theme.BG_CARD_HOVER : Theme.BG_INPUT);
        Surface.outline(g, getX(), getY(), width, height, hovered ? Theme.BORDER_STRONG : Theme.BORDER);

        Minecraft mc = Minecraft.getInstance();
        T current = value();
        Component text = current == null ? Component.empty() : label.apply(current);
        g.drawCenteredString(mc.font, text, getX() + width / 2, getY() + (height - 8) / 2, Theme.TEXT);

        // Стрелки по краям: без них поле неотличимо от поля ввода.
        int mid = getY() + (height - 8) / 2;
        g.drawString(mc.font, "<", getX() + 2 * Theme.GRID, mid, Theme.TEXT_MUTED, false);
        g.drawString(mc.font, ">", getX() + width - 3 * Theme.GRID, mid, Theme.TEXT_MUTED, false);
    }

    @Override
    protected void updateWidgetNarration(NarrationElementOutput output) {
        defaultButtonNarrationText(output);
    }
}
