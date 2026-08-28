package dev.noro.client.mixin;

import dev.noro.client.ui.Radial;
import net.minecraft.client.MouseHandler;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * While the radial menu is open the mouse moves the selection, not the head.
 *
 * <p>The delta has to be taken here: {@code MouseHandler} zeroes it right after
 * this call, and there is no other place it's still visible.
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
