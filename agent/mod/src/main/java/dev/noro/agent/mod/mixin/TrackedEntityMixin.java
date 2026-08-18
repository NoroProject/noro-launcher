package dev.noro.agent.mod.mixin;

import dev.noro.agent.mod.ModVanishManager;
import net.minecraft.server.level.ChunkMap;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Entity;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(targets = "net.minecraft.server.level.ChunkMap$TrackedEntity")
public abstract class TrackedEntityMixin {
    static {
        ModVanishManager.trackedEntityMixinLoaded = true;
    }
    @Shadow @Final
    Entity entity;

    @Inject(method = "updatePlayer", at = @At("HEAD"), cancellable = true)
    private void noro$hideVanishedEntity(ServerPlayer player, CallbackInfo ci) {
        if (entity instanceof ServerPlayer vanishedTarget) {
            if (ModVanishManager.getInstance().isVanished(vanishedTarget.getUUID())) {
                if (!ModVanishManager.getInstance().canSee(player, vanishedTarget)) {
                    ci.cancel();
                }
            }
        }
    }
}
