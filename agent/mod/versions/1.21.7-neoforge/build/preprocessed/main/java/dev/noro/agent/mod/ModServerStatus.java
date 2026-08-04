package dev.noro.agent.mod;

import dev.noro.agent.core.ServerStatus;
import net.minecraft.server.MinecraftServer;

/**
 * Цифры для heartbeat. Эти три метода {@code MinecraftServer} не менялись с 1.16,
 * поэтому файл общий для всех версий и лоадеров.
 */
record ModServerStatus(MinecraftServer server) implements ServerStatus {

    @Override
    public int online() {
        return server.getPlayerCount();
    }

    @Override
    public int maxPlayers() {
        return server.getMaxPlayers();
    }

    @Override
    public String version() {
        //#if FABRIC
        //$$ return "Fabric " + server.getServerVersion();
        //#else
        return "NeoForge " + server.getServerVersion();
        //#endif
    }
}
