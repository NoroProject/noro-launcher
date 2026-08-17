package dev.noro.agent.paper;

import dev.noro.agent.core.GameBridge;
import java.util.Collection;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.Collectors;
import net.kyori.adventure.text.Component;
import org.bukkit.Server;
import org.bukkit.entity.Player;
import org.bukkit.plugin.Plugin;

/** Модерация глазами Bukkit: кто в сети, кого отключить и что ему сказать. */
final class PaperBridge implements GameBridge {

    private final Plugin plugin;
    private final Server server;

    PaperBridge(Plugin plugin) {
        this.plugin = plugin;
        this.server = plugin.getServer();
    }

    @Override
    public Optional<UUID> onlineUuid(String name) {
        Player player = server.getPlayerExact(name);
        return Optional.ofNullable(player).map(Player::getUniqueId);
    }

    @Override
    public Collection<String> onlineNames() {
        return server.getOnlinePlayers().stream().map(Player::getName).collect(Collectors.toList());
    }

    @Override
    public void kick(UUID uuid, String message) {
        // Отключение обязано идти в главном потоке: наказание приходит из
        // потока WebSocket, а трогать сетевую сессию оттуда нельзя.
        onMain(uuid, player -> player.kick(text(message)));
    }

    @Override
    public void tell(UUID uuid, String message) {
        onMain(uuid, player -> player.sendMessage(text(message)));
    }

    @Override
    public void actionbar(UUID uuid, String message) {
        onMain(uuid, player -> player.sendActionBar(text(message)));
    }

    @Override
    public void announce(String message) {
        server.getScheduler().runTask(plugin, () -> server.broadcast(text(message)));
    }

    private void onMain(UUID uuid, java.util.function.Consumer<Player> action) {
        server.getScheduler().runTask(plugin, () -> {
            Player player = server.getPlayer(uuid);
            // Игрок мог выйти сам, пока кадр летел: это не ошибка.
            if (player != null) {
                action.accept(player);
            }
        });
    }

    /** Разбор разметки общий с модами: см. {@link PaperText}. */
    private static Component text(String message) {
        return PaperText.parse(message);
    }
}
