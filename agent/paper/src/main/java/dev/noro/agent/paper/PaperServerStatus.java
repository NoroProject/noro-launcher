package dev.noro.agent.paper;

import dev.noro.agent.core.ServerStatus;
import org.bukkit.Server;

/** Цифры для heartbeat со стороны Paper. */
record PaperServerStatus(Server server) implements ServerStatus {

    @Override
    public int online() {
        return server.getOnlinePlayers().size();
    }

    @Override
    public int maxPlayers() {
        return server.getMaxPlayers();
    }

    @Override
    public String version() {
        return "Paper " + server.getMinecraftVersion();
    }
}
