package dev.noro.agent.paper;

import dev.noro.agent.core.AccessGate;
import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.Moderation;
import dev.noro.agent.core.PermissionSet;
import dev.noro.agent.core.ProfileCache;
import dev.noro.agent.core.RoleApplier;
import java.util.UUID;
import net.kyori.adventure.text.Component;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.player.AsyncPlayerPreLoginEvent;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;
import org.slf4j.Logger;

/**
 * Проверка доступа до входа в мир.
 *
 * <p>{@code AsyncPlayerPreLoginEvent} — единственный момент, когда отказ выглядит
 * как отказ в подключении, а не как кик через секунду после появления в мире.
 * Событие уже асинхронное, поэтому и запрос к мастеру, и синхронизация групп
 * здесь блокирующие: главный поток они не задевают, а игрок входит с готовыми
 * группами, без мигания префикса.
 */
final class LoginListener implements Listener {

    private final AgentConfig config;
    private final MasterClient client;
    private final RoleApplier roleSync;
    private final PaperPermissions permissions;
    private final ProfileCache profiles;
    private final Moderation moderation;
    private final Logger log;

    LoginListener(
            AgentConfig config,
            MasterClient client,
            RoleApplier roleSync,
            PaperPermissions permissions,
            ProfileCache profiles,
            Moderation moderation,
            Logger log) {
        this.config = config;
        this.client = client;
        this.roleSync = roleSync;
        this.permissions = permissions;
        this.profiles = profiles;
        this.moderation = moderation;
        this.log = log;
    }

    @EventHandler(priority = EventPriority.HIGH)
    public void onPreLogin(AsyncPlayerPreLoginEvent event) {
        AccessGate.Decision decision = AccessGate.check(client, config, event.getUniqueId(), log);

        if (!decision.allowed()) {
            log.info("Denied {}: {}", event.getName(), decision.message());
            event.disallow(
                    AsyncPlayerPreLoginEvent.Result.KICK_OTHER, Component.text(decision.message()));
            return;
        }

        // profile пуст только в открытом режиме при недоступном мастере —
        // тогда синхронизировать просто нечем.
        if (decision.profile() == null) {
            return;
        }
        // Раньше, чем игрок появится в мире: дальше сеть трогать уже нельзя.
        permissions.remember(event.getUniqueId(), PermissionSet.of(decision.profile().permissions()));
        profiles.remember(event.getUniqueId(), decision.profile());
        if (roleSync != null) {
            roleSync.apply(event.getUniqueId(), decision.profile()).join();
        }
    }

    /**
     * Мут и непрочитанные предупреждения — уже после входа в мир: писать
     * игроку, который ещё на экране загрузки, некуда.
     */
    @EventHandler
    public void onJoin(PlayerJoinEvent event) {
        UUID uuid = event.getPlayer().getUniqueId();
        moderation.applier().greet(uuid, profiles.get(uuid));
    }

    /**
     * Профиль живёт ровно столько, сколько игрок на сервере: за него отвечает
     * тот же слушатель, который его и завёл.
     */
    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        UUID uuid = event.getPlayer().getUniqueId();
        profiles.forget(uuid);
        moderation.mutes().forget(uuid);
    }
}
