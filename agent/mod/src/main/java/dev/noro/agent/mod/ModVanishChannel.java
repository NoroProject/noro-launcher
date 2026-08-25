//#if NEOFORGE && MC>=12100
//$$ package dev.noro.agent.mod;
//$$
//$$ import io.netty.buffer.ByteBuf;
//$$ import java.util.ArrayList;
//$$ import java.util.List;
//$$ import java.util.UUID;
//$$ import net.minecraft.core.UUIDUtil;
//$$ import net.minecraft.network.codec.ByteBufCodecs;
//$$ import net.minecraft.network.codec.StreamCodec;
//$$ import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
//#if MC>=12111
//$$ import net.minecraft.resources.Identifier;
//#else
//$$ import net.minecraft.resources.ResourceLocation;
//#endif
//$$ import net.minecraft.server.MinecraftServer;
//$$ import net.minecraft.server.level.ServerPlayer;
//$$ import net.neoforged.bus.api.SubscribeEvent;
//$$ import net.neoforged.fml.common.EventBusSubscriber;
//$$ import net.neoforged.neoforge.network.PacketDistributor;
//$$ import net.neoforged.neoforge.network.event.RegisterPayloadHandlersEvent;
//$$
//$$ /**
//$$  * Кто сейчас в ванише — клиентскому моду.
//$$  *
//$$  * <p>Ваниш целиком серверный: миксины просто не отправляют сущность тем, кому
//$$  * её видеть нельзя. Поэтому клиент сам по себе не знает ни того, что он в
//$$  * ванише, ни того, что скрыт кто-то рядом, — а знать надо: иначе модератор
//$$  * забывает, в каком он состоянии, и разговаривает с игроком, будучи невидимым.
//$$  *
//$$  * <p>Список фильтруется по тому же {@code canSee}, что и сама видимость: кому
//$$  * нельзя видеть скрытого, тот не узнает о нём и отсюда. Право проверяет сервер,
//$$  * клиент только рисует.
//$$  *
//$$  * <p>Канал необязательный. Ванильный клиент и клиент без нашего мода про него
//$$  * не знают — им ничего не уходит, и подключиться это не мешает.
//$$  */
//$$ // Шины событий свели в одну в 1.21.4: до того регистрацию надо было явно
//$$ // просить на модовой, после — параметра `bus` больше нет.
        //#if MC>=12104
//$$ @EventBusSubscriber(modid = "noro_agent")
        //#else
//$$ @EventBusSubscriber(modid = "noro_agent", bus = EventBusSubscriber.Bus.MOD)
        //#endif
//$$ public final class ModVanishChannel {
//$$
//$$     public record Payload(List<UUID> vanished) implements CustomPacketPayload {
//$$
//$$         // В 1.21.11 ResourceLocation переименован в Identifier.
//#if MC>=12111
//$$         public static final CustomPacketPayload.Type<Payload> TYPE = new CustomPacketPayload.Type<>(
//$$                 Identifier.fromNamespaceAndPath("noro", "vanish"));
//#else
//$$         public static final CustomPacketPayload.Type<Payload> TYPE = new CustomPacketPayload.Type<>(
//$$                 ResourceLocation.fromNamespaceAndPath("noro", "vanish"));
//#endif
//$$
//$$         public static final StreamCodec<ByteBuf, Payload> CODEC = StreamCodec.composite(
//$$                 UUIDUtil.STREAM_CODEC.apply(ByteBufCodecs.list()), Payload::vanished, Payload::new);
//$$
//$$         @Override
//$$         public CustomPacketPayload.Type<? extends CustomPacketPayload> type() {
//$$             return TYPE;
//$$         }
//$$     }
//$$
//$$     private ModVanishChannel() {}
//$$
//$$     @SubscribeEvent
//$$     public static void register(RegisterPayloadHandlersEvent event) {
//$$         // Обработчика на сервере нет: канал односторонний. Клиент про ваниш
//$$         // ничего не решает — он его только показывает.
//$$         event.registrar("1").optional().playToClient(Payload.TYPE, Payload.CODEC, (payload, context) -> {});
//$$     }
//$$
//$$     /** Разослать всем: у каждого свой список — свой набор прав. */
//$$     public static void broadcast(MinecraftServer server) {
//$$         if (server == null) {
//$$             return;
//$$         }
//$$         for (ServerPlayer viewer : server.getPlayerList().getPlayers()) {
//$$             send(server, viewer);
//$$         }
//$$     }
//$$
//$$     /** Одному игроку: при заходе и при каждой смене состава. */
//$$     public static void send(MinecraftServer server, ServerPlayer viewer) {
//$$         if (server == null || !viewer.connection.hasChannel(Payload.TYPE)) {
//$$             return;
//$$         }
//$$         ModVanishManager vanish = ModVanishManager.getInstance();
//$$         List<UUID> visible = new ArrayList<>();
//$$         for (UUID uuid : vanish.getVanished()) {
//$$             ServerPlayer hidden = server.getPlayerList().getPlayer(uuid);
//$$             if (hidden != null && vanish.canSee(viewer, hidden)) {
//$$                 visible.add(uuid);
//$$             }
//$$         }
//$$         PacketDistributor.sendToPlayer(viewer, new Payload(visible));
//$$     }
//$$ }
//#endif
