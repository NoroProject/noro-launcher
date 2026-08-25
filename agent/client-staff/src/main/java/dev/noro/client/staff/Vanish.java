package dev.noro.client.staff;

import io.netty.buffer.ByteBuf;
import java.util.List;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import net.minecraft.client.Minecraft;
import net.minecraft.core.UUIDUtil;
import net.minecraft.network.codec.ByteBufCodecs;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.neoforge.network.event.RegisterPayloadHandlersEvent;

/**
 * Кто сейчас в ванише — по данным сервера, а не по догадке.
 *
 * <p>Ваниш серверный: клиент сам по себе не знает ни того, что скрыт он, ни
 * того, что скрыт кто-то рядом. Считать по «я же сам нажал {@code /vanish}»
 * нельзя — ваниш включают и командой руками, и автоматически при заходе, и
 * тогда панель врала бы ровно в тот момент, когда на неё смотрят.
 *
 * <p>Отбор по правам делает сервер: сюда приезжают только те, кого этому игроку
 * и так видно. Клиент ничего не решает, он рисует.
 */
@EventBusSubscriber(modid = NoroStaff.ID, bus = EventBusSubscriber.Bus.MOD)
public final class Vanish {

    /** Тот же канал, что у агента: имя и формат обязаны совпадать. */
    public record Payload(List<UUID> vanished) implements CustomPacketPayload {

        public static final CustomPacketPayload.Type<Payload> TYPE = new CustomPacketPayload.Type<>(
                ResourceLocation.fromNamespaceAndPath("noro", "vanish"));

        public static final StreamCodec<ByteBuf, Payload> CODEC = StreamCodec.composite(
                UUIDUtil.STREAM_CODEC.apply(ByteBufCodecs.list()), Payload::vanished, Payload::new);

        @Override
        public CustomPacketPayload.Type<? extends CustomPacketPayload> type() {
            return TYPE;
        }
    }

    private static final Set<UUID> HIDDEN = ConcurrentHashMap.newKeySet();

    private Vanish() {}

    @SubscribeEvent
    public static void register(RegisterPayloadHandlersEvent event) {
        // Необязательный: сервер без агента про канал не знает, и это не повод
        // не пускать игрока.
        event.registrar("1").optional().playToClient(Payload.TYPE, Payload.CODEC,
                (payload, context) -> context.enqueueWork(() -> accept(payload.vanished())));
    }

    private static void accept(List<UUID> vanished) {
        HIDDEN.clear();
        HIDDEN.addAll(vanished);
    }

    /** Забыть всё при отключении: на другом сервере состав свой. */
    public static void clear() {
        HIDDEN.clear();
    }

    public static boolean hidden(UUID uuid) {
        return uuid != null && HIDDEN.contains(uuid);
    }

    /** Скрыт ли сам игрок: по этому HUD решает, показывать ли предупреждение. */
    public static boolean self() {
        Minecraft mc = Minecraft.getInstance();
        return mc.player != null && hidden(mc.player.getUUID());
    }

    /** Сколько скрытых видно рядом — кроме себя. */
    public static int others() {
        Minecraft mc = Minecraft.getInstance();
        UUID self = mc.player == null ? null : mc.player.getUUID();
        int count = 0;
        for (UUID uuid : HIDDEN) {
            if (!uuid.equals(self)) {
                count++;
            }
        }
        return count;
    }
}
