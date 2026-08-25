package dev.noro.agent.mod;

import dev.noro.agent.core.AccessGate;
import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.ProfileCache;
import dev.noro.agent.core.RoleApplier;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;

/**
 * Вход игрока: доступ, роли, наказания.
 *
 * <p>Отдельно от {@link AgentRuntime}, потому что это единственное место, где
 * решение мастера превращается в отказ на экране, — и держать его рядом с
 * жизненным циклом мода значит смешивать две разные истории.
 */
final class ModJoin {

    private final AgentConfig config;
    private final MasterClient client;
    private final ProfileCache profiles;
    private final ModModeration moderation;
    private final dev.noro.agent.core.PrefixService prefixes;
    private final Runnable refreshPack;

    ModJoin(
            AgentConfig config,
            MasterClient client,
            ProfileCache profiles,
            ModModeration moderation,
            dev.noro.agent.core.PrefixService prefixes,
            Runnable refreshPack) {
        this.config = config;
        this.client = client;
        this.profiles = profiles;
        this.moderation = moderation;
        this.prefixes = prefixes;
        this.refreshPack = refreshPack;
    }

    void accept(MinecraftServer server, ServerPlayer player, AccessGate.Decision negotiated, RoleApplier roleSync) {
        UUID uuid = player.getUUID();
        // getScoreboardName(), а не getGameProfile().getName(): GameProfile стал
        // record в 1.21.9, и геттер там теперь name(). Имя нужно только для лога,
        // а этот метод одинаков на всём диапазоне.
        String name = player.getScoreboardName();

        // На Forge и NeoForge мастера уже спросили на логине — второй раз за тем
        // же ответом не ходим. На Fabric такой фазы нет, и решение снимается тут.
        if (negotiated != null) {
            apply(server, player, name, uuid, negotiated, roleSync);
            return;
        }
        // Запрос к мастеру уводим с главного потока, решение возвращаем на него:
        // MinecraftServer сам является Executor'ом.
        CompletableFuture.supplyAsync(() -> AccessGate.check(client, config, uuid, AgentRuntime.LOG))
                .thenAcceptAsync(decision -> apply(server, player, name, uuid, decision, roleSync), server);
    }

    private void apply(
            MinecraftServer server, ServerPlayer player, String name, UUID uuid, AccessGate.Decision decision, RoleApplier roleSync) {
        if (!decision.allowed()) {
            // Отказ приходит уже после входа в мир: пред-логин хука без микширования
            // на этих платформах нет. В прокси-топологии проверку надо ставить на
            // прокси, чтобы игрок вообще не доходил до бэкенда.
            AgentRuntime.LOG.info("Denied {}: {}", name, decision.denial().reason());
            // Единственный разрыв API на всём диапазоне 1.18.2 → 26.x:
            // Component.literal появился в 1.19, до него был TextComponent.
            player.connection.disconnect(ModText.parse(moderation.denialText(decision.denial())));
            return;
        }
        // Пустой профиль отсеивает сам кэш: мастер мог не ответить.
        profiles.remember(uuid, decision.profile());
        if (decision.profile() != null && decision.profile().vanishOnJoin()) {
            ModVanishManager.getInstance().setVanish(player, true, decision.profile().locale());
        }
        moderation.greet(uuid, decision.profile());
        if (roleSync != null && decision.profile() != null) {
            roleSync.apply(uuid, decision.profile());
        }
        // ВАЖНО: Права записаны в кэш! Теперь пересылаем дерево команд игроку,
        // чтобы Brigadier пересчитал requires() с загруженными правами мастера.
        if (server != null) {
            server.getPlayerList().sendPlayerPermissionLevel(player);
        }
        // Состав ваниша — вошедшему, и заново всем: если он вошёл скрытым, у
        // остальных список устарел ровно в этот момент.
        ModVanishManager.getInstance().announce();

        // Состав плашек мог смениться, пока сервер работает: роль правят в
        // админке, а не перезапуском. Спрашиваем при каждом входе — запрос
        // дешёвый, а выдача всё равно случится только при смене суммы.
        CompletableFuture.runAsync(refreshPack);

        // Пак при входе не выдаём вовсе: он приезжает игроку вместе со сборкой
        // лаунчера. Выдача сервера заставляет клиент перезагружать ресурсы, и
        // делать это на каждом входе — ради того, что у игрока уже есть, —
        // незачем. Остаётся один случай, когда выдать надо: состав плашек
        // поменяли, пока игрок в сети, — этим занимается refreshPrefixPack.
    }

    /**
     * Пришёл ли игрок с нашими модами.
     *
     * <p>По согласованному каналу: ядро клиента едет в каждой сборке лаунчера,
     * поэтому канал есть ровно у тех, у кого пак и так стоит. На платформах без
     * канала считаем, что модов нет, — тогда пак выдаётся, как и раньше.
     */
    private static boolean hasOurMods(ServerPlayer player) {
        //#if NEOFORGE && MC>=12100
        //$$ return ModHelloChannel.present(player);
        //#else
        return false;
        //#endif
    }
}
