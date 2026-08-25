package dev.noro.client.staff;

import dev.noro.client.ui.Theme;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.player.Player;

/**
 * Числа слежки: то, чего не даст ни сайт, ни глаза.
 *
 * <p>«Развернулся на 178° за два тика четырнадцать раз» — это запись в деле, а
 * не ощущение. Показания приблизительные: клиент видит цель через трекинг,
 * позиции приходят пачками и сглажены. Они помогают смотреть, вердикт выносит
 * человек — поэтому и в дело едут словами модератора, а не сами по себе.
 */
public final class Telemetry {

    /** За сколько тиков считается разворот. Два — минимум, который видно. */
    private static final int SNAP_TICKS = 2;

    /** Резкий разворот: меньше — это обычная игра, больше — уже редкость. */
    private static final float SNAP_DEGREES = 90f;

    private static Player target;
    private static float lastYaw;
    private static double lastX;
    private static double lastY;
    private static double lastZ;
    private static int ticks;
    private static double speed;
    private static double height;
    private static int snaps;

    private Telemetry() {}

    public static void watch(Player player) {
        target = player;
        snaps = 0;
        ticks = 0;
    }

    public static void stop() {
        target = null;
    }

    /** Раз в тик: скорость, высота над землёй и резкие развороты. */
    public static void tick() {
        Player p = target;
        if (p == null) {
            return;
        }
        double dx = p.getX() - lastX;
        double dz = p.getZ() - lastZ;
        speed = Math.sqrt(dx * dx + dz * dz) * 20;
        height = p.getY() - lastY >= 0 ? p.getY() - groundBelow(p) : height;

        if (ticks % SNAP_TICKS == 0) {
            float turn = Math.abs(net.minecraft.util.Mth.wrapDegrees(p.getYRot() - lastYaw));
            if (turn >= SNAP_DEGREES) {
                snaps++;
            }
            lastYaw = p.getYRot();
        }
        lastX = p.getX();
        lastY = p.getY();
        lastZ = p.getZ();
        ticks++;
    }

    private static double groundBelow(Player p) {
        return p.level().getHeight(
                net.minecraft.world.level.levelgen.Heightmap.Types.MOTION_BLOCKING,
                p.getBlockX(),
                p.getBlockZ());
    }

    /** Есть ли за кем следить: по этому HUD решает, нужна ли ему секция чисел. */
    public static boolean watching() {
        return target != null;
    }

    public static void render(GuiGraphics g, Font font, int x, int y) {
        if (target == null) {
            return;
        }
        // Без тени: на плашке HUD она размазывает мелкий шрифт, а читать эти
        // три строки приходится на ходу.
        g.drawString(
                font,
                Component.translatable("noro.cases.telemetry.speed", String.format("%.1f", speed)),
                x,
                y,
                Theme.TEXT_MUTED,
                false);
        g.drawString(
                font,
                Component.translatable("noro.cases.telemetry.height", String.format("%.1f", height)),
                x,
                y + 10,
                Theme.TEXT_MUTED,
                false);
        g.drawString(
                font,
                Component.translatable("noro.cases.telemetry.snaps", snaps),
                x,
                y + 20,
                snaps > 0 ? Theme.ERROR : Theme.TEXT_MUTED,
                false);
    }

    /** Сводка одной строкой — её модератор и приложит заметкой к делу. */
    public static String summary() {
        return String.format(
                "speed %.1f, height %.1f, snaps %d", speed, height, snaps);
    }
}
