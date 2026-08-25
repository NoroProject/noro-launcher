package dev.noro.client.mixin;

import dev.noro.client.ui.Radial;
import net.minecraft.client.MouseHandler;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Пока открыто радиальное меню, мышь двигает выбор, а не голову.
 *
 * <p>Иначе прицеливаться приходится поворотом вида: работает, но модератор в
 * ванише крутится на месте, а после закрытия камеру надо возвращать рывком.
 *
 * <p>Накопленную дельту забираем здесь же и отменяем поворот: {@code
 * MouseHandler} обнуляет её сразу после этого вызова, и другого места, где её
 * ещё видно, нет.
 */
@Mixin(MouseHandler.class)
public abstract class MouseHandlerMixin {

    @Shadow
    private double accumulatedDX;

    @Shadow
    private double accumulatedDY;

    @Inject(method = "turnPlayer", at = @At("HEAD"), cancellable = true)
    private void noro$aimInsteadOfTurning(CallbackInfo ci) {
        if (Radial.grabsMouse()) {
            Radial.aimWith(accumulatedDX, accumulatedDY);
            ci.cancel();
        }
    }
}
