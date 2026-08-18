package dev.noro.agent.paper;

import dev.noro.agent.core.AccessGate;
import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.AgentEvents;
import dev.noro.agent.core.DenialScreen;
import dev.noro.agent.core.IpHash;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.Moderation;
import dev.noro.agent.core.PermissionSet;
import dev.noro.agent.core.ProfileCache;
import dev.noro.agent.core.RoleApplier;
import java.net.InetSocketAddress;
import java.util.UUID;
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
    private final AgentEvents events;
    private final Logger log;

    private final VanishManager vanishManager;

    LoginListener(
            AgentConfig config,
            MasterClient client,
            RoleApplier roleSync,
            PaperPermissions permissions,
            ProfileCache profiles,
            Moderation moderation,
            VanishManager vanishManager,
            AgentEvents events,
            Logger log) {
        this.config = config;
        this.client = client;
        this.roleSync = roleSync;
        this.permissions = permissions;
        this.profiles = profiles;
        this.moderation = moderation;
        this.vanishManager = vanishManager;
        this.events = events;
        this.log = log;
    }

    @EventHandler(priority = EventPriority.HIGH)
    public void onPreLogin(AsyncPlayerPreLoginEvent event) {
        AccessGate.Decision decision = AccessGate.check(client, config, event.getUniqueId(), log);

        if (!decision.allowed()) {
            log.info("Denied {}: {}", event.getName(), decision.denial().reason());
            // Текст рисуем тем же путём, что и кик при живом бане: разметка
            // шаблона иначе доехала бы до игрока сырой, с «&l» и «#f87171».
            String screen = DenialScreen.text(
                    decision.denial(), moderation.templates(), moderation.rules());
            event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_OTHER, PaperText.parse(screen));
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
        dev.noro.agent.core.PlayerProfile profile = profiles.get(uuid);
        if (profile != null && profile.vanishOnJoin()) {
            vanishManager.setVanish(event.getPlayer(), true, profile.locale());
        }
        moderation.applier().greet(uuid, profile);
        events.playerJoin(uuid, addressHash(event), vanishManager.isVanished(uuid));
    }

    /** Адрес уходит только хешем: сравнивать им можно, читать — нечего. */
    private String addressHash(PlayerJoinEvent event) {
        InetSocketAddress address = event.getPlayer().getAddress();
        return address == null ? null : IpHash.of(address.getHostString(), config.secret());
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
        events.playerLeave(uuid, "quit");
    }
}
