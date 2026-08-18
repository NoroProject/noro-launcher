package dev.noro.agent.mod.mixin;

import dev.noro.agent.mod.ModVanishManager;
import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.entity.player.Player;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ItemEntity.class)
public abstract class ItemEntityMixin {
    static {
        ModVanishManager.itemEntityMixinLoaded = true;
    }
    @Inject(method = "playerTouch", at = @At("HEAD"), cancellable = true)
    private void noro$preventVanishPickup(Player player, CallbackInfo ci) {
        if (ModVanishManager.getInstance().isVanished(player.getUUID())) {
            ci.cancel();
        }
    }
}
