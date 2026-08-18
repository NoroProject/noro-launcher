package dev.noro.agent.paper;

import dev.noro.agent.core.PlayerProfile;
import dev.noro.agent.core.ProfileCache;
import java.util.UUID;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.block.BlockBreakEvent;
import org.bukkit.event.block.BlockPlaceEvent;
import org.bukkit.event.entity.EntityDamageByEntityEvent;
import org.bukkit.event.player.PlayerCommandPreprocessEvent;
import org.bukkit.event.player.PlayerMoveEvent;

/**
 * Ограничение действий замороженного игрока в игре: запрет движения, команд и урона.
 * Чат остаётся доступным для общения с модератором.
 */
final class FreezeListener implements Listener {

    private final ProfileCache profiles;

    FreezeListener(ProfileCache profiles) {
        this.profiles = profiles;
    }

    private boolean isFrozen(UUID uuid) {
        PlayerProfile profile = profiles.get(uuid);
        return profile != null && profile.frozen();
    }

    private String lang(UUID uuid) {
        PlayerProfile profile = profiles.get(uuid);
        return profile == null ? null : profile.locale();
    }

    @EventHandler(priority = EventPriority.LOWEST, ignoreCancelled = true)
    public void onMove(PlayerMoveEvent event) {
        if (event.getFrom().getBlockX() != event.getTo().getBlockX()
                || event.getFrom().getBlockY() != event.getTo().getBlockY()
                || event.getFrom().getBlockZ() != event.getTo().getBlockZ()) {
            UUID uuid = event.getPlayer().getUniqueId();
            if (isFrozen(uuid)) {
                event.setCancelled(true);
                String msg = dev.noro.agent.core.AgentStrings.get(lang(uuid), "freeze_actionbar", "§cYou are frozen by a moderator!");
                event.getPlayer().sendActionBar(PaperText.parse(msg));
            }
        }
    }

    @EventHandler(priority = EventPriority.LOWEST, ignoreCancelled = true)
    public void onCommand(PlayerCommandPreprocessEvent event) {
        UUID uuid = event.getPlayer().getUniqueId();
        if (isFrozen(uuid)) {
            event.setCancelled(true);
            String msg = dev.noro.agent.core.AgentStrings.get(lang(uuid), "freeze_no_cmd", "§cYou are frozen and cannot use commands.");
            event.getPlayer().sendMessage(PaperText.parse(msg));
        }
    }

    @EventHandler(priority = EventPriority.LOWEST, ignoreCancelled = true)
    public void onDamage(EntityDamageByEntityEvent event) {
        if (event.getDamager() instanceof Player attacker && isFrozen(attacker.getUniqueId())) {
            event.setCancelled(true);
            UUID uuid = attacker.getUniqueId();
            String msg = dev.noro.agent.core.AgentStrings.get(lang(uuid), "freeze_no_damage", "§cYou are frozen and cannot deal damage.");
            attacker.sendMessage(PaperText.parse(msg));
            return;
        }
        if (event.getEntity() instanceof Player victim && isFrozen(victim.getUniqueId())) {
            event.setCancelled(true);
        }
    }

    @EventHandler(priority = EventPriority.LOWEST, ignoreCancelled = true)
    public void onBreak(BlockBreakEvent event) {
        if (isFrozen(event.getPlayer().getUniqueId())) {
            event.setCancelled(true);
        }
    }

    @EventHandler(priority = EventPriority.LOWEST, ignoreCancelled = true)
    public void onPlace(BlockPlaceEvent event) {
        if (isFrozen(event.getPlayer().getUniqueId())) {
            event.setCancelled(true);
        }
    }
}
