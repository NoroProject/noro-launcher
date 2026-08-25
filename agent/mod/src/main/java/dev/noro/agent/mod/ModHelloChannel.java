//#if NEOFORGE && MC>=12100
//$$ package dev.noro.agent.mod;
//$$
//$$ import io.netty.buffer.ByteBuf;
//$$ import net.minecraft.network.codec.StreamCodec;
//$$ import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
//#if MC>=12111
//$$ import net.minecraft.resources.Identifier;
//#else
//$$ import net.minecraft.resources.ResourceLocation;
//#endif
//$$ import net.minecraft.server.level.ServerPlayer;
//$$ import net.neoforged.bus.api.SubscribeEvent;
//$$ import net.neoforged.fml.common.EventBusSubscriber;
//$$ import net.neoforged.neoforge.network.event.RegisterPayloadHandlersEvent;
//$$
//$$ /**
//$$  * Отметка «клиент пришёл с нашими модами».
//$$  *
//$$  * <p>Ничего не передаёт: важен сам факт, что канал согласован. Регистрирует его
//$$  * с той стороны ядро клиента, а оно едет в каждой сборке лаунчера, — значит
//$$  * канал есть ровно у тех, у кого пак с плашками и так лежит локально.
//$$  *
//$$  * <p>Нужно это, чтобы не выдавать таким игрокам пак второй раз. Выдача сервера
//$$  * заставляет клиент перезагружать ресурсы на каждом входе, а у пришедшего из
//$$  * лаунчера пак уже стоит и загружен при запуске игры.
//$$  */
        //#if MC>=12104
//$$ @EventBusSubscriber(modid = "noro_agent")
        //#else
//$$ @EventBusSubscriber(modid = "noro_agent", bus = EventBusSubscriber.Bus.MOD)
        //#endif
//$$ public final class ModHelloChannel {
//$$
//$$     public record Payload() implements CustomPacketPayload {
//#if MC>=12111
//$$         public static final CustomPacketPayload.Type<Payload> TYPE = new CustomPacketPayload.Type<>(
//$$                 Identifier.fromNamespaceAndPath("noro", "hello"));
//#else
//$$         public static final CustomPacketPayload.Type<Payload> TYPE = new CustomPacketPayload.Type<>(
//$$                 ResourceLocation.fromNamespaceAndPath("noro", "hello"));
//#endif
//$$
//$$         public static final StreamCodec<ByteBuf, Payload> CODEC =
//$$                 StreamCodec.unit(new Payload());
//$$
//$$         @Override
//$$         public CustomPacketPayload.Type<? extends CustomPacketPayload> type() {
//$$             return TYPE;
//$$         }
//$$     }
//$$
//$$     private ModHelloChannel() {}
//$$
//$$     @SubscribeEvent
//$$     public static void register(RegisterPayloadHandlersEvent event) {
//$$         event.registrar("1").optional().playToClient(Payload.TYPE, Payload.CODEC, (payload, context) -> {});
//$$     }
//$$
//$$     /** Есть ли у клиента наши моды — а значит и пак с плашками. */
//$$     public static boolean present(ServerPlayer player) {
//$$         return player.connection.hasChannel(Payload.TYPE);
//$$     }
//$$ }
//#endif
