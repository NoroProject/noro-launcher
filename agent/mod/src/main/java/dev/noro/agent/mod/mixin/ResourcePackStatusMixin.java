//#if MC>=12003
//$$ package dev.noro.agent.mod.mixin;
//$$
//$$ import dev.noro.agent.mod.AgentRuntime;
//$$ import net.minecraft.network.protocol.common.ServerboundResourcePackPacket;
//$$ import net.minecraft.server.network.ServerCommonPacketListenerImpl;
//$$ import org.spongepowered.asm.mixin.Mixin;
//$$ import org.spongepowered.asm.mixin.injection.At;
//$$ import org.spongepowered.asm.mixin.injection.Inject;
//$$ import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
//$$
//$$ /**
//$$  * Ответ клиента на выданный пак: принял или нет.
//$$  *
//$$  * <p>Знать это обязательно. Плашка роли — картинка из пака, и тому, кто пак не
//$$  * принял, тот же символ показался бы белым квадратом. Считать по «мы же ему
//$$  * отправили» нельзя: пак ещё и не докачивается.
//$$  *
//$$  * <p>Миксином, потому что события на это у лоадеров нет. Необязательный, как и
//$$  * остальные здесь: не легло — плашек просто не будет, сервер работает дальше.
//$$  */
//$$ @Mixin(ServerCommonPacketListenerImpl.class)
//$$ public abstract class ResourcePackStatusMixin {
//$$
//$$     @Inject(method = "handleResourcePackResponse", at = @At("HEAD"))
//$$     private void noro$rememberPackStatus(ServerboundResourcePackPacket packet, CallbackInfo ci) {
//$$         AgentRuntime runtime = AgentRuntime.current();
//$$         if (runtime == null) {
//$$             return;
//$$         }
//$$         ServerCommonPacketListenerImpl self = (ServerCommonPacketListenerImpl) (Object) this;
//$$         if (!(self instanceof net.minecraft.server.network.ServerGamePacketListenerImpl game)) {
//$$             return;
//$$         }
//$$         // Успех — это только «загрузил и применил». Всё прочее (отказ, битая
//$$         // закачка, откат) значит, что картинок у игрока нет.
//$$         boolean ok = packet.action()
//$$                 == ServerboundResourcePackPacket.Action.SUCCESSFULLY_LOADED;
//$$         runtime.prefixes.status(game.player.getUUID(), ok);
//$$     }
//$$ }
//#endif
