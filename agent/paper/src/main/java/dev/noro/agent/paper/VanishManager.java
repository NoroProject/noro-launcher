package dev.noro.agent.paper;

import dev.noro.agent.core.AgentStrings;
import dev.noro.agent.core.NoroAgentApi;
import dev.noro.agent.core.PlayerProfile;
import dev.noro.agent.core.RoleInfo;
import java.util.Collections;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;
import org.bukkit.Bukkit;
import org.bukkit.entity.Player;
import org.bukkit.plugin.Plugin;

public class VanishManager {
    public static final String PERM_SEE = "noro.mod.vanish.see";
    public static final String PERM_USE = "noro.mod.vanish.use";
    public static final String PERM_STAFF = "noro.mod.staff.notify";

    private final Plugin plugin;
    private final Set<UUID> vanished = ConcurrentHashMap.newKeySet();

    public VanishManager(Plugin plugin) {
        this.plugin = plugin;
    }

    public boolean isVanished(UUID uuid) {
        return vanished.contains(uuid);
    }

    public Set<UUID> getVanished() {
        return Collections.unmodifiableSet(vanished);
    }

    public boolean canSee(Player viewer, Player target) {
        if (viewer.equals(target)) return true;
        if (!viewer.hasPermission(PERM_SEE)) return false;
        int viewerWeight = getMaxRoleWeight(viewer.getUniqueId());
        int targetWeight = getMaxRoleWeight(target.getUniqueId());
        return viewerWeight >= targetWeight;
    }

    public int getMaxRoleWeight(UUID uuid) {
        PlayerProfile profile = NoroAgentApi.profile(uuid);
        if (profile == null || profile.roles() == null || profile.roles().isEmpty()) return 0;
        int max = 0;
        for (RoleInfo role : profile.roles()) {
            if (role.sortOrder() > max) {
                max = role.sortOrder();
            }
        }
        return max;
    }

    public void toggleVanish(Player player, String lang) {
        setVanish(player, !isVanished(player.getUniqueId()), lang);
    }

    public void setVanish(Player player, boolean enable, String lang) {
        UUID uuid = player.getUniqueId();
        if (enable) {
            vanished.add(uuid);
            for (Player other : Bukkit.getOnlinePlayers()) {
                if (other.equals(player)) continue;
                if (!canSee(other, player)) {
                    other.hidePlayer(plugin, player);
                    // Фейковый выход для тех, кто не видит модератора
                    other.sendMessage(Component.translatable("multiplayer.player.left", NamedTextColor.YELLOW, Component.text(player.getName())));
                }
            }
            player.sendMessage(AgentStrings.get(lang, "vanish_enabled"));
        } else {
            vanished.remove(uuid);
            for (Player other : Bukkit.getOnlinePlayers()) {
                if (other.equals(player)) continue;
                other.showPlayer(plugin, player);
                if (!canSee(other, player)) {
                    // Фейковый вход при выключении ваниша
                    other.sendMessage(Component.translatable("multiplayer.player.joined", NamedTextColor.YELLOW, Component.text(player.getName())));
                }
            }
            player.sendMessage(AgentStrings.get(lang, "vanish_disabled"));
        }
    }

    public void sendStaffChat(Player vanishedPlayer, String rawMessage) {
        Component formatted = Component.text("§8[§cVANISH§8] §f" + vanishedPlayer.getName() + "§7: §f" + rawMessage);
        vanishedPlayer.sendMessage(formatted);
        for (Player other : Bukkit.getOnlinePlayers()) {
            if (other.equals(vanishedPlayer)) continue;
            if (other.hasPermission(PERM_STAFF) || canSee(other, vanishedPlayer)) {
                other.sendMessage(formatted);
            }
        }
    }

    public void sendVanishList(Player sender) {
        if (!sender.hasPermission(PERM_SEE)) {
            sender.sendMessage(AgentStrings.get(null, "no_perm_view"));
            return;
        }
        StringBuilder sb = new StringBuilder("§7Скрытые модераторы (§f").append(vanished.size()).append("§7): ");
        boolean first = true;
        for (UUID vUuid : vanished) {
            Player p = Bukkit.getPlayer(vUuid);
            if (p != null) {
                if (!first) sb.append("§7, ");
                sb.append("§f").append(p.getName());
                first = false;
            }
        }
        if (first) sb.append("§8(нет)");
        sender.sendMessage(sb.toString());
    }

    public void onPlayerJoin(Player newPlayer) {
        if (isVanished(newPlayer.getUniqueId())) {
            // Если заходящий уже в ванише — скрываем его от обычных игроков
            for (Player other : Bukkit.getOnlinePlayers()) {
                if (!other.equals(newPlayer) && !canSee(other, newPlayer)) {
                    other.hidePlayer(plugin, newPlayer);
                }
            }
        }
        // Скрываем всех скрытых от заходящего игрока, если у него нет прав/веса
        for (UUID vUuid : vanished) {
            Player vPlayer = Bukkit.getPlayer(vUuid);
            if (vPlayer != null && !canSee(newPlayer, vPlayer)) {
                newPlayer.hidePlayer(plugin, vPlayer);
            }
        }
    }
}
