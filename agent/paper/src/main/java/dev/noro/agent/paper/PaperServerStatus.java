package dev.noro.agent.paper;

import dev.noro.agent.core.ServerStatus;
import dev.noro.agent.core.TickMeter;
import java.util.Collection;
import java.util.List;
import java.util.UUID;
import org.bukkit.Server;
import org.bukkit.entity.Player;

/**
 * Цифры для heartbeat со стороны Paper.
 *
 * <p>Тики считает {@link TickMeter}, а не {@code Bukkit.getTPS()}: своим
 * счётчиком одинаково меряется весь диапазон версий, а заодно видно зависание —
 * у вставшего сервера обновлять число некому.
 */
record PaperServerStatus(Server server, TickMeter meter, VanishManager vanishManager) implements ServerStatus {

    @Override
    public int online() {
        return Math.max(0, server.getOnlinePlayers().size() - vanishManager.getVanished().size());
    }

    @Override
    public int maxPlayers() {
        return server.getMaxPlayers();
    }

    @Override
    public String version() {
        return "Paper " + server.getMinecraftVersion();
    }

    @Override
    public Collection<UUID> players() {
        // Копией, а не видом: список игроков читает фоновый поток heartbeat, и
        // ходить по живой коллекции сервера оттуда нельзя.
        return List.copyOf(server.getOnlinePlayers().stream()
                .map(Player::getUniqueId)
                .filter(u -> !vanishManager.isVanished(u))
                .toList());
    }

    @Override
    public Collection<UUID> vanished() {
        return List.copyOf(vanishManager.getVanished());
    }

    @Override
    public Double tps() {
        return meter.tps();
    }

    @Override
    public int stalledSeconds() {
        return meter.stalledSeconds();
    }
}
