package dev.noro.client;

import io.netty.buffer.ByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.neoforge.network.event.RegisterPayloadHandlersEvent;

/**
 * Отметка для сервера: у этого клиента наши моды.
 *
 * <p>Ничего не передаёт — важен сам факт, что канал согласован. Ядро едет в
 * каждой сборке лаунчера, поэтому канал есть ровно у тех, у кого вместе со
 * сборкой приехал и пак с плашками.
 *
 * <p>По нему сервер понимает, что выдавать пак не надо: он уже стоит и загружен
 * при запуске игры. Иначе клиент перезагружал бы ресурсы на каждом входе — ради
 * того, что у него и так есть.
 */
@EventBusSubscriber(modid = NoroCore.ID, bus = EventBusSubscriber.Bus.MOD)
public final class Hello {

    public record Payload() implements CustomPacketPayload {

        public static final CustomPacketPayload.Type<Payload> TYPE = new CustomPacketPayload.Type<>(
                ResourceLocation.fromNamespaceAndPath("noro", "hello"));

        public static final StreamCodec<ByteBuf, Payload> CODEC = StreamCodec.unit(new Payload());

        @Override
        public CustomPacketPayload.Type<? extends CustomPacketPayload> type() {
            return TYPE;
        }
    }

    private Hello() {}

    @SubscribeEvent
    public static void register(RegisterPayloadHandlersEvent event) {
        // Необязательный: сервер без агента про канал не знает, и это не повод
        // не пускать игрока.
        event.registrar("1").optional().playToClient(Payload.TYPE, Payload.CODEC, (payload, context) -> {});
    }
}
