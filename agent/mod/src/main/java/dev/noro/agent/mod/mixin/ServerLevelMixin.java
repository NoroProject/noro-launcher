package dev.noro.agent.mod.mixin;

import dev.noro.agent.mod.ModVanishManager;
import java.util.List;
import java.util.function.Predicate;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(ServerLevel.class)
public abstract class ServerLevelMixin {
    static {
        ModVanishManager.serverLevelMixinLoaded = true;
    }
    @Inject(method = "getPlayers(Ljava/util/function/Predicate;)Ljava/util/List;", at = @At("RETURN"), cancellable = true)
    private void noro$filterVanishedPlayers(Predicate<? super ServerPlayer> predicate, CallbackInfoReturnable<List<ServerPlayer>> cir) {
        List<ServerPlayer> list = cir.getReturnValue();
        if (list != null && !list.isEmpty()) {
            List<ServerPlayer> filtered = list.stream()
                    .filter(p -> !ModVanishManager.getInstance().isVanished(p.getUUID()))
                    .toList();
            cir.setReturnValue(filtered);
        }
    }
}
