package dev.noro.client.staff.mixin;

import dev.noro.client.staff.Vanish;
import net.minecraft.client.renderer.entity.LivingEntityRenderer;
import net.minecraft.world.entity.LivingEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Скрытый ванишем — полупрозрачный, как невидимка сам себе.
 *
 * <p>Через {@code isBodyVisible}, а не своей отрисовкой: ваниль уже умеет
 * рисовать «тело не видно, но смотрящему показать» — ровно так игрок под
 * зельем невидимости видит себя от третьего лица. Мы говорим ей, что тело
 * невидимо, и она сама рисует модель с той же прозрачностью. Своя отрисовка
 * повторяла бы слои брони, шляпы и накидки — и разъезжалась бы с ванилью на
 * каждом обновлении.
 *
 * <p>Утечки нет: сущность скрытого игрока сервер вообще не отправляет тем, кому
 * его видеть нельзя, а список приезжает уже отфильтрованным по правам. Кто досюда
 * дошёл, тот и так его видит — вопрос лишь в том, отличит ли он его от обычного
 * игрока.
 */
@Mixin(LivingEntityRenderer.class)
public abstract class LivingEntityRendererMixin {

    @Inject(method = "isBodyVisible", at = @At("HEAD"), cancellable = true)
    private void noro$fadeVanished(LivingEntity entity, CallbackInfoReturnable<Boolean> info) {
        if (Vanish.hidden(entity.getUUID())) {
            info.setReturnValue(false);
        }
    }
}
