package dev.noro.agent.paper;

import dev.noro.agent.core.PermissionSet;
import java.util.HashMap;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;
import org.bukkit.permissions.Permission;
import org.bukkit.permissions.PermissionAttachment;
import org.bukkit.plugin.Plugin;

/**
 * Права мастера на стороне Paper — через {@link PermissionAttachment}.
 *
 * <p>Права снимаются раньше, чем выдаются: в сеть ходит пред-логин, а вешать
 * attachment можно только на живого {@code Player}. Между этими моментами набор
 * и лежит в {@link #pending}.
 */
final class PaperPermissions implements Listener {

    private final Plugin plugin;

    /** Пишет асинхронный пред-логин, читает главный поток на входе игрока. */
    private final Map<UUID, PermissionSet> pending = new ConcurrentHashMap<>();

    /** Только главный поток: вход и выход игрока приходят синхронно. */
    private final Map<UUID, PermissionAttachment> attached = new HashMap<>();

    PaperPermissions(Plugin plugin) {
        this.plugin = plugin;
    }

    void remember(UUID uuid, PermissionSet permissions) {
        pending.put(uuid, permissions);
    }

    /**
     * {@code LOWEST}, чтобы права были на месте раньше, чем другие плагины
     * начнут их спрашивать в своих обработчиках входа.
     */
    @EventHandler(priority = EventPriority.LOWEST)
    public void onJoin(PlayerJoinEvent event) {
        Player player = event.getPlayer();
        PermissionSet permissions = pending.remove(player.getUniqueId());
        if (permissions == null) {
            return;
        }
        PermissionAttachment attachment = player.addAttachment(plugin);
        attached.put(player.getUniqueId(), attachment);
        grant(attachment, permissions);
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        UUID uuid = event.getPlayer().getUniqueId();
        pending.remove(uuid);
        PermissionAttachment attachment = attached.remove(uuid);
        if (attachment != null) {
            attachment.remove();
        }
    }

    /**
     * Bukkit шаблоны не раскрывает: {@code hasPermission} сверяет имя точно, и
     * узел {@code noro.admin.*} сам по себе не даст {@code noro.admin.users}.
     * Поэтому шаблон разворачивается по тем правам, которые зарегистрированы на
     * сервере, — большего списка тут всё равно не существует.
     *
     * <p>Сам шаблон тоже выдаётся: часть плагинов проверяет именно его.
     */
    private void grant(PermissionAttachment attachment, PermissionSet permissions) {
        for (String pattern : permissions.patterns()) {
            attachment.setPermission(pattern, true);
            if (!pattern.endsWith("*")) {
                continue;
            }
            for (Permission known : plugin.getServer().getPluginManager().getPermissions()) {
                if (PermissionSet.matches(pattern, known.getName())) {
                    attachment.setPermission(known.getName(), true);
                }
            }
        }
    }
}
