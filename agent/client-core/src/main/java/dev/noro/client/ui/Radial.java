package dev.noro.client.ui;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Колесо действий: зажал клавишу, повёл мышью, отпустил.
 *
 * <p>Кольцо, а не список подписей по кругу: сектор — крупная мишень, попасть в
 * него можно небрежным движением, а список требует прицеливания. Список кнопок
 * проигрывает колесу, когда действие повторяется десятки раз за смену.
 *
 * <p>Дырка в середине — не украшение: это зона отмены. Отпустил, не уводя
 * мышь, — ничего не произошло, и это должно быть видно, а не угадываться.
 *
 * <p>Собирается так:
 * <pre>{@code
 * Radial wheel = Radial.of()
 *     .item("tp", () -> Actions.teleport(id))
 *     .item("freeze", () -> Actions.freeze(id));
 * // в тике:      wheel.hold(KEY.isDown(), caseIsOpen);
 * // в отрисовке: wheel.render(graphics);
 * }</pre>
 */
public final class Radial {

    private static final int INNER = 10 * Theme.GRID;
    private static final int OUTER = 26 * Theme.GRID;

    /** Зазор между секторами, чтобы кольцо не выглядело сплошным блином. */
    private static final double GAP = Math.toRadians(1.5);

    /** Во сколько раз ход мыши мельче пикселя кольца. */
    private static final double AIM_SCALE = 0.4;

    /**
     * Открыто ли колесо у кого-нибудь.
     *
     * <p>Статикой, потому что спрашивает это миксин на {@code MouseHandler}: у
     * него нет ссылки ни на одно колесо, а знать нужно одно — держать ли голову.
     */
    private static volatile Radial grabbing;

    public static boolean grabsMouse() {
        return grabbing != null;
    }

    /** Ход мыши за кадр от миксина: его же он у игры и отобрал. */
    public static void aimWith(double dx, double dy) {
        Radial wheel = grabbing;
        if (wheel != null) {
            wheel.feed(dx, dy);
        }
    }

    private final List<Item> items = new ArrayList<>();
    private boolean open;
    private int hovered = -1;
    private double aimX;
    private double aimY;

    private record Item(Component label, Runnable action, boolean enabled) {}

    private Radial() {}

    public static Radial of() {
        return new Radial();
    }

    /** Пункт колеса. Ключ перевода, а не готовый текст. */
    public Radial item(String translationKey, Runnable action) {
        return item(translationKey, action, true);
    }

    /**
     * Пункт, которого может не быть.
     *
     * <p>{@code enabled} гасит его, а не прячет: колесо запоминается положением
     * секторов, и исчезающий пункт сдвигает все остальные.
     */
    public Radial item(String translationKey, Runnable action, boolean enabled) {
        items.add(new Item(Component.translatable(translationKey), action, enabled));
        return this;
    }

    public boolean open() {
        return open;
    }

    /**
     * Состояние клавиши на этот тик. Отпустили — выполняем то, на чём стояли.
     *
     * @param down зажата ли клавиша
     * @param allowed можно ли открывать (нет дела — нет и колеса)
     */
    public void hold(boolean down, boolean allowed) {
        if (down) {
            if (!open && allowed && !items.isEmpty()) {
                open = true;
                grabbing = this;
                hovered = -1;
                aimX = 0;
                aimY = 0;
            }
            return;
        }
        if (open) {
            open = false;
            grabbing = null;
            if (hovered >= 0 && hovered < items.size() && items.get(hovered).enabled()) {
                items.get(hovered).action().run();
            }
            hovered = -1;
        }
    }

    /**
     * Ход мыши за кадр — его отдаёт миксин, перехвативший поворот вида.
     *
     * <p>Голова при этом стоит: движение целиком уходит в выбор сектора, и
     * возвращать камеру на место после закрытия больше не нужно.
     */
    private void feed(double dx, double dy) {
        aimX += dx * AIM_SCALE;
        aimY += dy * AIM_SCALE;
        aim();
    }

    private void aim() {
        double distance = Math.sqrt(aimX * aimX + aimY * aimY);
        if (distance < INNER) {
            hovered = -1;
            return;
        }
        // Не даём улететь за кольцо: иначе возврат к центру требует того же
        // длинного хода мышью обратно.
        if (distance > OUTER) {
            aimX *= OUTER / distance;
            aimY *= OUTER / distance;
        }
        double angle = Math.atan2(aimX, -aimY);
        if (angle < 0) {
            angle += Math.PI * 2;
        }
        double sector = Math.PI * 2 / items.size();
        hovered = (int) (((angle + sector / 2) % (Math.PI * 2)) / sector);
    }

    public void render(GuiGraphics g) {
        if (!open) {
            return;
        }
        Minecraft mc = Minecraft.getInstance();
        float cx = mc.getWindow().getGuiScaledWidth() / 2f;
        float cy = mc.getWindow().getGuiScaledHeight() / 2f;
        double sector = Math.PI * 2 / items.size();

        for (int i = 0; i < items.size(); i++) {
            Item item = items.get(i);
            double from = i * sector - sector / 2 + GAP;
            double to = from + sector - 2 * GAP;
            int fill = !item.enabled()
                    ? Theme.RING_DISABLED
                    : i == hovered ? Theme.RING_ACTIVE : Theme.RING;
            Ring.sector(g, cx, cy, INNER, OUTER, from, to, fill);

            double mid = i * sector;
            float lx = cx + (float) (Math.sin(mid) * (INNER + OUTER) / 2.0);
            float ly = cy - (float) (Math.cos(mid) * (INNER + OUTER) / 2.0);
            int color = !item.enabled()
                    ? Theme.TEXT_MUTED
                    : i == hovered ? Theme.ON_CTA : Theme.TEXT;
            g.drawCenteredString(mc.font, item.label(), (int) lx, (int) ly - 4, color);
        }

        // Подпись в дырке: что произойдёт, если отпустить прямо сейчас.
        Component centre = hovered < 0
                ? Component.translatable("noro.ui.radial.cancel")
                : items.get(hovered).label();
        g.drawCenteredString(mc.font, centre, (int) cx, (int) cy - 4,
                hovered < 0 ? Theme.TEXT_MUTED : Theme.CTA);
    }
}
