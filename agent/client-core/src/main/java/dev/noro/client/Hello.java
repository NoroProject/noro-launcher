package dev.noro.client;

import io.netty.buffer.ByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.neoforge.network.event.RegisterPayloadHandlersEvent;

/**
 * Tells the server this client has our mods. Carries no data — what matters is
 * that the channel was negotiated at all.
 *
 * <p>The core ships in every launcher build, and so does the prefix pack, so a
 * client with this channel already has the pack loaded. The server uses that to
 * skip sending it; otherwise the client would reload resources on every join for
 * something it already has.
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
        // Optional: a server without the agent doesn't know this channel, and
        // that's no reason to refuse the player.
        event.registrar("1").optional().playToClient(Payload.TYPE, Payload.CODEC, (payload, context) -> {});
    }
}
