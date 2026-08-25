package dev.noro.client.staff;

import dev.noro.client.staff.ui.DossierHud;
import dev.noro.client.staff.ui.VanishBadge;
import dev.noro.client.staff.ui.WatchOverlay;
import net.minecraft.client.Minecraft;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.neoforge.client.event.ClientTickEvent;
import net.neoforged.neoforge.client.event.RenderGuiEvent;

/**
 * Отрисовка поверх игры и опрос телеметрии.
 *
 * <p>Ничего не выдаёт разбор: панель не издаёт звуков, не тратит слот
 * интерфейса и не мигает частицами. Модератор в ванише должен оставаться
 * невидимым и на экране тоже.
 */
@EventBusSubscriber(modid = NoroStaff.ID, value = Dist.CLIENT)
public final class StaffHud {

    private StaffHud() {}

    @SubscribeEvent
    public static void onRender(RenderGuiEvent.Post event) {
        Minecraft mc = Minecraft.getInstance();
        if (mc.player == null || mc.options.hideGui) {
            return;
        }
        int width = mc.getWindow().getGuiScaledWidth();
        int height = mc.getWindow().getGuiScaledHeight();
        WatchOverlay.render(event.getGuiGraphics(), width, height);
        DossierHud.render(event.getGuiGraphics(), width, height);
        VanishBadge.render(event.getGuiGraphics(), width, height);
    }

    /** Телеметрия слежки считается по тикам: реже — и разворот не поймать. */
    @SubscribeEvent
    public static void onTick(ClientTickEvent.Post event) {
        Minecraft mc = Minecraft.getInstance();
        if (mc.player == null) {
            return;
        }
        Telemetry.tick();
    }
}
