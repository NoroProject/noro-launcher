package dev.noro.agent.mod;

import dev.noro.agent.core.AccessGate;
import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.HeartbeatTask;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.LuckPermsSupport;
import dev.noro.agent.core.RoleApplier;
import java.nio.file.Path;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Общая часть модов Fabric и NeoForge.
 *
 * <p>Ни одной директивы препроцессора здесь нет и быть не должно: все вызовы
 * Minecraft, которые нужны агенту, на обеих платформах одинаковы. Расходятся
 * только точки входа и способ узнать каталог конфигурации — они и живут в
 * {@code FabricEntry} и {@code NeoForgeEntry}.
 */
public final class AgentRuntime {

    public static final Logger LOG = LoggerFactory.getLogger("noro-agent");

    private AgentConfig config;
    private MasterClient client;
    private RoleApplier roleSync;
    private HeartbeatTask heartbeat;
    private ModPermissions permissions;

    /**
     * Сервер запоминаем со старта, а не спрашиваем у игрока: {@code ServerPlayer.getServer()}
     * исчез в 1.21.9, и лезть за ним через сущность — лишняя точка отказа на каждой
     * новой версии. Со старта он приходит одинаково везде.
     */
    private MinecraftServer server;

    /** @return false, если агент не настроен и включаться ему не с чем */
    public boolean init(Path configDir) {
        try {
            config = AgentConfig.load(configDir.resolve("noro-agent.properties"));
        } catch (RuntimeException e) {
            LOG.error("Cannot start: {}", e.getMessage());
            return false;
        }
        LOG.info("Starting against {}", config);
        client = new MasterClient(config);
        permissions = new ModPermissions(client, config);
        return true;
    }

    /** Общее хранилище прав: точка входа отдаёт его обработчику своего лоадера. */
    ModPermissions permissions() {
        return permissions;
    }

    public void onServerStarted(MinecraftServer server) {
        this.server = server;
        // LuckPerms грузится тоже модом, поэтому спрашиваем его после старта
        // сервера, а не в инициализации — там порядок не гарантирован.
        roleSync = LuckPermsSupport.tryCreate(LOG);
        heartbeat = new HeartbeatTask(client, new ModServerStatus(server), config, LOG);
        heartbeat.start();
        // Каталог узлов уходит мимо главного потока: старт сервера не должен
        // ждать сеть ради подсказки в админке.
        CompletableFuture.runAsync(permissions::report);
    }

    /**
     * Права снимаются на фазе логина, до входа в мир.
     *
     * <p>Именно на тайминге ломается LuckPerms: он грузит данные игрока на
     * {@code PlayerLoggedInEvent}, а тот срабатывает уже после
     * {@code sendPlayerPermissionLevel} → {@code sendCommands}, где права
     * читают, — и игрок улетает с «Invalid player data».
     */
    public void onPlayerNegotiation(UUID uuid) {
        permissions.load(uuid);
    }

    /** Права живут ровно столько, сколько игрок на сервере. */
    public void onPlayerLeave(UUID uuid) {
        permissions.forget(uuid);
    }

    public void onServerStopping() {
        if (heartbeat != null) {
            heartbeat.close();
        }
    }

    public void onPlayerJoin(ServerPlayer player) {
        if (server == null) {
            return;
        }
        UUID uuid = player.getUUID();
        // getScoreboardName(), а не getGameProfile().getName(): GameProfile стал
        // record в 1.21.9, и геттер там теперь name(). Имя нужно только для лога,
        // а этот метод одинаков на всём диапазоне.
        String name = player.getScoreboardName();

        // На Forge и NeoForge мастера уже спросили на логине — второй раз за тем
        // же ответом не ходим. На Fabric такой фазы нет, и решение снимается тут.
        AccessGate.Decision negotiated = permissions.takeDecision(uuid);
        if (negotiated != null) {
            apply(player, name, uuid, negotiated);
            return;
        }
        // Запрос к мастеру уводим с главного потока, решение возвращаем на него:
        // MinecraftServer сам является Executor'ом.
        CompletableFuture.supplyAsync(() -> AccessGate.check(client, config, uuid, LOG))
                .thenAcceptAsync(decision -> apply(player, name, uuid, decision), server);
    }

    private void apply(ServerPlayer player, String name, UUID uuid, AccessGate.Decision decision) {
        if (!decision.allowed()) {
            // Отказ приходит уже после входа в мир: пред-логин хука без микширования
            // на этих платформах нет. В прокси-топологии проверку надо ставить на
            // прокси, чтобы игрок вообще не доходил до бэкенда.
            LOG.info("Denied {}: {}", name, decision.message());
            // Единственный разрыв API на всём диапазоне 1.18.2 → 26.x:
            // Component.literal появился в 1.19, до него был TextComponent.
            //#if MC>=11900
            //$$ player.connection.disconnect(Component.literal(decision.message()));
            //#else
            player.connection.disconnect(new net.minecraft.network.chat.TextComponent(decision.message()));
            //#endif
            return;
        }
        if (roleSync != null && decision.profile() != null) {
            roleSync.apply(uuid, decision.profile());
        }
    }
}
