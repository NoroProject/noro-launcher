//#if FORGE
package dev.noro.agent.mod;

import java.util.UUID;
import net.minecraft.server.level.ServerPlayer;
import net.minecraftforge.common.MinecraftForge;
import net.minecraftforge.event.entity.player.PlayerEvent;
import net.minecraftforge.event.entity.player.PlayerNegotiationEvent;
import net.minecraftforge.event.server.ServerStartedEvent;
import net.minecraftforge.event.server.ServerStoppingEvent;
import net.minecraftforge.eventbus.api.SubscribeEvent;
import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.fml.loading.FMLPaths;
import net.minecraftforge.server.permission.events.PermissionGatherEvent;

/**
 * Точка входа Forge — для 1.18.2–1.20.1, где NeoForge ещё не существовало.
 * Весь файл под {@code //#if FORGE}: на остальных сборках он закомментирован.
 *
 * <p>От NeoForge отличается только пакетами и шиной событий, поэтому вся логика
 * по-прежнему в {@link AgentRuntime}, общем для всех трёх лоадеров.
 */
@Mod("noro_agent")
public final class ForgeEntry {

    private final AgentRuntime runtime = new AgentRuntime();

    public ForgeEntry() {
        if (!runtime.init(FMLPaths.CONFIGDIR.get())) {
            return;
        }
        MinecraftForge.EVENT_BUS.register(this);
    }

    @SubscribeEvent
    public void onServerStarted(ServerStartedEvent event) {
        runtime.onServerStarted(event.getServer());
        NoroPermissionHandler.warnIfInactive(AgentRuntime.LOG);
    }

    @SubscribeEvent
    public void onServerStopping(ServerStoppingEvent event) {
        runtime.onServerStopping();
    }

    /** Оба Gather-события приходят на старте сервера, до входа первого игрока. */
    @SubscribeEvent
    public void onGatherHandler(PermissionGatherEvent.Handler event) {
        event.addPermissionHandler(
                NoroPermissionHandler.IDENTIFIER,
                nodes -> new NoroPermissionHandler(nodes, runtime.permissions()));
    }

    @SubscribeEvent
    public void onGatherNodes(PermissionGatherEvent.Nodes event) {
        runtime.permissions().rememberNodes(NoroPermissionHandler.names(event.getNodes()));
    }

    /**
     * Единственная фаза, где можно сходить в сеть и всё же успеть: логин ждёт
     * работу из {@code enqueueWork}, а {@code PlayerLoggedInEvent} наступает уже
     * после того, как сервер прочитал права игрока.
     */
    @SubscribeEvent
    public void onNegotiation(PlayerNegotiationEvent event) {
        UUID uuid = ProfileId.of(event.getProfile());
        event.enqueueWork(() -> runtime.onPlayerNegotiation(uuid));
    }

    // getEntity(), а не getPlayer(): первый есть и в 1.18.2 (он объявлен ещё в
    // EntityEvent), а getPlayer() исчез в 1.19. Проверка instanceof всё равно
    // сужает тип, так что разницы в коде нет и директива не нужна.
    @SubscribeEvent
    public void onPlayerLoggedIn(PlayerEvent.PlayerLoggedInEvent event) {
        if (event.getEntity() instanceof ServerPlayer player) {
            runtime.onPlayerJoin(player);
        }
    }

    @SubscribeEvent
    public void onPlayerLoggedOut(PlayerEvent.PlayerLoggedOutEvent event) {
        runtime.onPlayerLeave(event.getEntity().getUUID());
    }
}
//#endif
