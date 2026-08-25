package dev.noro.client.staff;

import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.neoforge.client.event.RenderLivingEvent;

/**
 * Подсветка цели через стены — то, чего серверный агент не смог.
 *
 * <p>{@code GameBridge.glow} честно возвращает false: свечение для одного
 * зрителя требует подмены пакета, а ванильный {@code setGlowing} светит всем и
 * выдаёт разбор нарушителю. Клиент рисует контур локально.
 *
 * <p>Горит, только пока мастер подтвердил активное дело за этим модератором, и
 * гаснет вместе с ним. Новых позиций мод не запрашивает — рисуются лишь те
 * сущности, которые клиент и так получил от сервера.
 */
@EventBusSubscriber(modid = NoroStaff.ID, value = Dist.CLIENT)
public final class Highlight {

    private Highlight() {}

    /** Кого подсвечивать: цель открытого дела и никого больше. */
    public static boolean shouldGlow(Entity entity) {
        CaseModels.View view = NoroStaff.state().open();
        if (view == null || view.brief() == null || !(entity instanceof Player)) {
            return false;
        }
        String status = view.brief().status();
        if (!"in_review".equals(status)) {
            return false;
        }
        String target = view.brief().target_name();
        return target != null && target.equals(entity.getName().getString());
    }

    /**
     * Ванильное свечение переиспользуется как есть: контур рисуется только у
     * нас, потому что флаг ставится на клиенте и на сервер не уходит.
     */
    @SubscribeEvent
    public static void onRender(RenderLivingEvent.Pre<?, ?> event) {
        if (Minecraft.getInstance().player == null) {
            return;
        }
        Entity entity = event.getEntity();
        if (shouldGlow(entity)) {
            entity.setGlowingTag(true);
        } else if (entity.hasGlowingTag()) {
            entity.setGlowingTag(false);
        }
    }
}
