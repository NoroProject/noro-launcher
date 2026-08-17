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

    ModJoin(AgentConfig config, MasterClient client, ProfileCache profiles, ModModeration moderation) {
        this.config = config;
        this.client = client;
        this.profiles = profiles;
        this.moderation = moderation;
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
            AgentRuntime.LOG.info("Denied {}: {}", name, decision.message());
            // Единственный разрыв API на всём диапазоне 1.18.2 → 26.x:
            // Component.literal появился в 1.19, до него был TextComponent.
            player.connection.disconnect(ModText.parse(decision.message()));
            return;
        }
        // Пустой профиль отсеивает сам кэш: мастер мог не ответить.
        profiles.remember(uuid, decision.profile());
        moderation.greet(uuid, decision.profile());
        if (roleSync != null && decision.profile() != null) {
            roleSync.apply(uuid, decision.profile());
        }
        // ВАЖНО: Права записаны в кэш! Теперь пересылаем дерево команд игроку,
        // чтобы Brigadier пересчитал requires() с загруженными правами мастера.
        if (server != null) {
            server.getPlayerList().sendPlayerPermissionLevel(player);
        }
    }
}
