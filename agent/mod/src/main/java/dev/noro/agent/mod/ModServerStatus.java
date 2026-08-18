package dev.noro.agent.mod;

import dev.noro.agent.core.ServerStatus;
import dev.noro.agent.core.TickMeter;
import java.util.Collection;
import java.util.List;
import java.util.UUID;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;

/**
 * Цифры для heartbeat. Эти три метода {@code MinecraftServer} не менялись с 1.16,
 * поэтому файл общий для всех версий и лоадеров.
 */
record ModServerStatus(MinecraftServer server, TickMeter meter) implements ServerStatus {

    @Override
    public int online() {
        int vanishedCount = ModVanishManager.getInstance().getVanished().size();
        return Math.max(0, server.getPlayerCount() - vanishedCount);
    }

    @Override
    public int maxPlayers() {
        return server.getMaxPlayers();
    }

    @Override
    public Collection<UUID> players() {
        // Копией, а не видом: список читает фоновый поток heartbeat, а сервер
        // правит его в игровом — обход живой коллекции оттуда развалится.
        return List.copyOf(server.getPlayerList().getPlayers().stream()
                .map(ServerPlayer::getUUID)
                .filter(u -> !ModVanishManager.getInstance().isVanished(u))
                .toList());
    }

    @Override
    public Collection<UUID> vanished() {
        return List.copyOf(ModVanishManager.getInstance().getVanished());
    }

    @Override
    public Double tps() {
        return meter.tps();
    }

    @Override
    public Double mspt() {
        return meter.mspt();
    }

    @Override
    public int stalledSeconds() {
        return meter.stalledSeconds();
    }

    @Override
    public String version() {
        //#if FABRIC
        return "Fabric " + server.getServerVersion();
        //#else
        //$$ return "NeoForge " + server.getServerVersion();
        //#endif
    }
}
