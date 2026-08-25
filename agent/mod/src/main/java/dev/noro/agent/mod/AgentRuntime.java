package dev.noro.agent.mod;

import com.mojang.brigadier.CommandDispatcher;
import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.HeartbeatTask;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.MasterHttp;
import dev.noro.agent.core.LuckPermsSupport;
import dev.noro.agent.core.NoroAgentApi;
import dev.noro.agent.core.PlayerProfile;
import dev.noro.agent.core.ProfileCache;
import dev.noro.agent.core.ProfileRefresher;
import dev.noro.agent.core.RoleApplier;
import dev.noro.agent.core.TickMeter;
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
    private ModPresence presence;

    /** Общий с {@link NoroAgentApi}: чужие моды читают профиль оттуда же. */
    private final ProfileCache profiles = NoroAgentApi.cache();

    /**
     * Счётчик тиков. Заполняет его точка входа своего лоадера — хук конца тика
     * у Fabric и NeoForge называется по-разному, а всё остальное одинаково.
     */
    private final TickMeter meter = new TickMeter();

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
        join = new ModJoin(config, client, profiles, moderation, prefixes, this::refreshPrefixPack);
        presence = new ModPresence(moderation);
        return true;
    }

    /** Счётчик тиков: точка входа подключает к нему хук своего лоадера. */
    public TickMeter meter() {
        return meter;
    }

    /** Общее хранилище прав: точка входа отдаёт его обработчику своего лоадера. */
    ModPermissions permissions() {
        return permissions;
    }

    /**
     * Работающий агент — миксинам, у которых своего пути к нему нет.
     *
     * <p>Одна на процесс: агент в игре ровно один, второго быть не может.
     */
    private static volatile AgentRuntime current;

    public static AgentRuntime current() {
        return current;
    }

    /** Плашки ролей: что рисовать и кто согласился их видеть. */
    public final dev.noro.agent.core.PrefixService prefixes = new dev.noro.agent.core.PrefixService();

    /**
     * Перечитать состав пака у мастера и раздать его тем, кто уже в сети.
     *
     * <p>Зовётся при старте и после правки ролей: пак меняется вместе с ними, и
     * ждать перезахода игрока незачем — клиент применяет выданный пак на ходу.
     */
    public void refreshPrefixPack() {
        String had = prefixes.pack().sha1();
        try {
            prefixes.pack(client.prefixPack());
        } catch (Exception e) {
            LOG.warn("Prefix pack is unavailable, roles keep their text prefixes: {}", e.toString());
            return;
        }
        ModBridge bridge = moderation.bridge();
        if (!prefixes.usable() || server == null || bridge == null) {
            return;
        }
        var pack = prefixes.pack();
        // Только если состав правда сменился. Иначе каждый опрос мастера гнал бы
        // игрокам один и тот же пак, а с ним и перезагрузку ресурсов.
        if (pack.sha1().equals(had)) {
            return;
        }
        for (ServerPlayer player : server.getPlayerList().getPlayers()) {
            // Пришедшему из лаунчера пак не выдаём: у него он приехал вместе со
            // сборкой, а смену подхватит живая синхронизация лаунчера — молча,
            // без ванильного окна «скачать набор?». Остальным окно неизбежно:
            // другого способа отдать пак чужому клиенту нет.
            //#if NEOFORGE && MC>=12100
            //$$ if (ModHelloChannel.present(player)) {
            //$$     continue;
            //$$ }
            //#endif
            bridge.sendResourcePack(player.getUUID(), pack.url(), pack.sha1());
        }
    }

    public void onServerStarted(MinecraftServer server) {
        this.server = server;
        current = this;
        dev.noro.agent.core.NoroAgentApi.attachPrefixes(prefixes);
        // LuckPerms грузится тоже модом, поэтому спрашиваем его после старта
        // сервера, а не в инициализации — там порядок не гарантирован.
        roleSync = LuckPermsSupport.tryCreate(LOG);
        ModServerStatus status = new ModServerStatus(server, meter);
        heartbeat = new HeartbeatTask(client, status, config, LOG);
        // Роль, выданную на сайте, игрок получает сейчас, а не после
        // перезахода: кадр profile_changed приводит нас сюда.
        moderation.attachRefresher(new ProfileRefresher(
                client, status::players, this::reapply, LOG));
        // Тоже после старта и по той же причине: Text Placeholder API — мод,
        // и до этого момента его может не быть в пути классов.
        ModPlaceholders.register(profiles);
        // Состав плашек — мимо главного потока: старт сервера не должен ждать
        // мастера, а без плашек сервер прекрасно работает.
        java.util.concurrent.CompletableFuture.runAsync(this::refreshPrefixPack);
        // Каталог узлов уходит мимо главного потока: старт сервера не должен
        // ждать сеть ради подсказки в админке.
        CompletableFuture.runAsync(permissions::report);

        // Модерация оживает только вместе с сервером: раньше кикать и писать в
        // чат было бы некому. Канал поднимается там же — и только после этого
        // сигналу жизни есть куда сообщить о зависании.
        moderation.start(server);
        heartbeat.reportStallsTo(moderation.events());
        heartbeat.start();
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

    public boolean checkChatMessage(ServerPlayer player, String rawText) {
        if (silenced(player)) {
            return true;
        }
        // Буфер держит окно разговора, из которого потом соберётся срез для
        // дела. Пишем до фильтров: заблокированное сообщение как раз и есть то,
        // что интересно разбору.
        if (moderation != null) {
            moderation.chatRing().message(player.getUUID(), player.getScoreboardName(), "public", rawText);
        }
        if (moderation != null) {
            dev.noro.agent.core.automod.ChatFilters.Result res = moderation.checkChatMessage(player.getUUID(), rawText);
            if (res.action() == dev.noro.agent.core.automod.ChatFilters.Action.DENY
                    || res.action() == dev.noro.agent.core.automod.ChatFilters.Action.PUNISH
                    || res.action() == dev.noro.agent.core.automod.ChatFilters.Action.ESCALATE) {
                ModText.send(player, ModText.parse("#f87171Message blocked by AutoMod [" + res.filterType() + "]"));
                return true;
            }
        }
        return false;
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

    /**
     * Обновлённый профиль: права в свою карту, группы — в LuckPerms.
     *
     * <p>Дерево команд игроку не пересылаем: Brigadier пересчитывает
     * {@code requires()} на своём такте, а внеплановая рассылка на сотню
     * игроков разом — заметный пакет каждому из них.
     */
    private void reapply(UUID uuid, PlayerProfile profile) {
        permissions.remember(uuid, profile);
        if (roleSync != null) {
            roleSync.apply(uuid, profile);
        }
    }

    /** Права и профиль живут ровно столько, сколько игрок на сервере. */
    public void onPlayerLeave(UUID uuid) {
        permissions.forget(uuid);
        profiles.forget(uuid);
        moderation.forget(uuid);
        presence.left(uuid);
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
        presence.joined(player);
    }
}
