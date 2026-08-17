package dev.noro.agent.mod;

import dev.noro.agent.core.GameBridge;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.UUID;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;

/**
 * Модерация глазами Minecraft: кто в сети, кого отключить и что ему сказать.
 *
 * <p>Один класс на Fabric, Forge и NeoForge — все вызовы отсюда есть на всех
 * трёх. Расходятся только версии игры, и это разведено препроцессором.
 */
final class ModBridge implements GameBridge {

    private final MinecraftServer server;

    ModBridge(MinecraftServer server) {
        this.server = server;
    }

    @Override
    public Optional<UUID> onlineUuid(String name) {
        ServerPlayer player = server.getPlayerList().getPlayerByName(name);
        return Optional.ofNullable(player).map(ServerPlayer::getUUID);
    }

    @Override
    public Collection<String> onlineNames() {
        List<ServerPlayer> players = server.getPlayerList().getPlayers();
        Collection<String> names = new ArrayList<>(players.size());
        for (ServerPlayer player : players) {
            names.add(player.getScoreboardName());
        }
        return names;
    }

    @Override
    public void kick(UUID uuid, String message) {
        // Наказание приходит из потока WebSocket, а сетевую сессию игрока можно
        // трогать только из главного. MinecraftServer сам является Executor'ом.
        onMain(uuid, player -> player.connection.disconnect(text(message)));
    }

    @Override
    public void tell(UUID uuid, String message) {
        onMain(uuid, player -> send(player, text(message)));
    }

    @Override
    public void actionbar(UUID uuid, String message) {
        onMain(uuid, player -> {
            //#if MC>=260000
            //$$ player.sendSystemMessage(text(message));
            //#else
            player.displayClientMessage(text(message), true);
            //#endif
        });
    }

    @Override
    public void announce(String message) {
        server.execute(() -> {
            Component component = text(message);
            for (ServerPlayer player : server.getPlayerList().getPlayers()) {
                send(player, component);
            }
        });
    }

    private void onMain(UUID uuid, java.util.function.Consumer<ServerPlayer> action) {
        server.execute(() -> {
            ServerPlayer player = server.getPlayerList().getPlayer(uuid);
            // Игрок мог выйти сам, пока кадр летел: это не ошибка.
            if (player != null) {
                action.accept(player);
            }
        });
    }

    /**
     * Системное сообщение игроку.
     *
     * <p>До 1.19 у сообщений был отправитель, и системные слал «никто» —
     * {@code Util.NIL_UUID}. С приходом подписанного чата параметр исчез.
     */
    private static void send(ServerPlayer player, Component component) {
        //#if MC>=11900
        player.sendSystemMessage(component);
        //#else
        //$$ player.sendMessage(component, net.minecraft.Util.NIL_UUID);
        //#endif
    }

    /**
     * Строка с {@code §}-кодами. Ванильный рендер разбирает их сам, поэтому
     * разбирать цвета в стили здесь незачем — и это единственный формат,
     * одинаковый на всём диапазоне версий.
     */
    private static Component text(String message) {
        return ModText.parse(message);
    }
}
