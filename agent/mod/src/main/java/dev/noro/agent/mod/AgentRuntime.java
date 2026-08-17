package dev.noro.agent.mod;

import com.mojang.brigadier.CommandDispatcher;
import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.HeartbeatTask;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.MasterHttp;
import dev.noro.agent.core.LuckPermsSupport;
import dev.noro.agent.core.NoroAgentApi;
import dev.noro.agent.core.ProfileCache;
import dev.noro.agent.core.RoleApplier;
import java.nio.file.Path;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import net.minecraft.commands.CommandSourceStack;
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
    private MasterHttp http;
    private MasterClient client;
    private RoleApplier roleSync;
    private HeartbeatTask heartbeat;
    private ModPermissions permissions;
    private ModModeration moderation;
    private ModJoin join;

    /** Общий с {@link NoroAgentApi}: чужие моды читают профиль оттуда же. */
    private final ProfileCache profiles = NoroAgentApi.cache();

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
        http = new MasterHttp(config);
        client = new MasterClient(http);
        permissions = new ModPermissions(client, config);
        moderation = new ModModeration(http, client, LOG);
        join = new ModJoin(config, client, profiles, moderation);
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
        // Тоже после старта и по той же причине: Text Placeholder API — мод,
        // и до этого момента его может не быть в пути классов.
        ModPlaceholders.register(profiles);
        // Каталог узлов уходит мимо главного потока: старт сервера не должен
        // ждать сеть ради подсказки в админке.
        CompletableFuture.runAsync(permissions::report);

        // Модерация оживает только вместе с сервером: раньше кикать и писать в
        // чат было бы некому.
        moderation.start(server);
    }

    /** Дерево команд собирается на каждом лоадере своим событием. */
    public void registerCommands(CommandDispatcher<CommandSourceStack> dispatcher) {
        moderation.registerCommands(dispatcher);
    }

    /**
     * Сказал ли замученный то, чего ему нельзя. Отказ игроку показан внутри.
     */
    public boolean silenced(ServerPlayer player) {
        return moderation != null && moderation.silenced(player);
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

    /** Права и профиль живут ровно столько, сколько игрок на сервере. */
    public void onPlayerLeave(UUID uuid) {
        permissions.forget(uuid);
        profiles.forget(uuid);
        moderation.forget(uuid);
    }

    public void onServerStopping() {
        if (heartbeat != null) {
            heartbeat.close();
        }
        if (moderation != null) {
            moderation.close();
        }
    }

    public void onPlayerJoin(ServerPlayer player) {
        if (server == null) {
            return;
        }
        join.accept(server, player, permissions.takeDecision(player.getUUID()), roleSync);
    }
}
