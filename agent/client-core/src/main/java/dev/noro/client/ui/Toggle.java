package dev.noro.client.ui;

import java.util.function.Consumer;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.network.chat.Component;

/**
 * Переключатель: дорожка и бегунок, как {@code AtomToggle} на сайте.
 *
 * <p>Состояние не меняется локально по клику. Пока сервер не подтвердил, тумблер
 * обязан показывать то, что реально лежит там, — иначе интерфейс врёт при
 * упавшем запросе. Владелец вызывает {@link #set(boolean)}, когда ответ пришёл.
 */
public final class Toggle extends AbstractButton {

    private static final int TRACK_W = 8 * Theme.GRID;
    private static final int TRACK_H = 4 * Theme.GRID;

    private final Consumer<Boolean> onChange;
    private boolean on;
    private boolean busy;

    private Toggle(int x, int y, int w, Component label, boolean on, Consumer<Boolean> onChange) {
        super(x, y, w, TRACK_H, label);
        this.on = on;
        this.onChange = onChange;
    }

    public static Toggle of(
            int x, int y, int w, Component label, boolean on, Consumer<Boolean> onChange) {
        return new Toggle(x, y, w, label, on, onChange);
    }

    public boolean on() {
        return on;
    }

    /** Состояние с сервера. Снимает ожидание: ответ пришёл. */
    public void set(boolean value) {
        this.on = value;
        this.busy = false;
    }

    @Override
    public void onPress() {
        if (busy) {
            return;
        }
        // Ждём подтверждения: показываем прежнее состояние, но приглушённо.
        busy = true;
        onChange.accept(!on);
    }

    @Override
    protected void renderWidget(GuiGraphics g, int mouseX, int mouseY, float partial) {
        int trackX = getX() + width - TRACK_W;
        int trackY = getY();
        int track = on ? Theme.SUCCESS : Theme.BG_INPUT;
        g.fill(trackX, trackY, trackX + TRACK_W, trackY + TRACK_H, busy ? Theme.BG_CARD : track);
        Surface.outline(g, trackX, trackY, TRACK_W, TRACK_H, Theme.BORDER);

        int thumb = on ? trackX + TRACK_W - TRACK_H : trackX;
        g.fill(
                thumb + 1,
                trackY + 1,
                thumb + TRACK_H - 1,
                trackY + TRACK_H - 1,
                busy ? Theme.TEXT_MUTED : Theme.TEXT);

        g.drawString(
                Minecraft.getInstance().font,
                getMessage(),
                getX(),
                trackY + (TRACK_H - 8) / 2,
                busy ? Theme.TEXT_MUTED : Theme.TEXT_SECONDARY);
    }

    @Override
    protected void updateWidgetNarration(NarrationElementOutput output) {
        defaultButtonNarrationText(output);
    }
}
