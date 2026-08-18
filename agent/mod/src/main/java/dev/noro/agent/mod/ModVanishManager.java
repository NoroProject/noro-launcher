package dev.noro.agent.mod;

import dev.noro.agent.core.AgentStrings;
import dev.noro.agent.core.NoroAgentApi;
import dev.noro.agent.core.PlayerProfile;
import dev.noro.agent.core.RoleInfo;
import java.util.Collections;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;

public class ModVanishManager {
    public static final String PERM_SEE = "noro.mod.vanish.see";
    public static final String PERM_USE = "noro.mod.vanish.use";
    public static final String PERM_STAFF = "noro.mod.staff.notify";

    public static volatile boolean trackedEntityMixinLoaded = false;
    public static volatile boolean serverLevelMixinLoaded = false;
    public static volatile boolean itemEntityMixinLoaded = false;

    private static final ModVanishManager INSTANCE = new ModVanishManager();
    private final Set<UUID> vanished = ConcurrentHashMap.newKeySet();
    private volatile MinecraftServer server;

    public static boolean isVanishFullySupported() {
        return trackedEntityMixinLoaded && serverLevelMixinLoaded && itemEntityMixinLoaded;
    }

    public static ModVanishManager getInstance() {
        return INSTANCE;
    }

    public void setServer(MinecraftServer server) {
        this.server = server;
    }

    public boolean isVanished(UUID uuid) {
        return uuid != null && vanished.contains(uuid);
    }

    public Set<UUID> getVanished() {
        return Collections.unmodifiableSet(vanished);
    }

    public boolean hasPermission(UUID uuid, String perm) {
        PlayerProfile profile = NoroAgentApi.profile(uuid);
        return profile != null && profile.permissions() != null && profile.permissions().contains(perm);
    }

    public boolean canSee(ServerPlayer viewer, ServerPlayer target) {
        if (viewer.getUUID().equals(target.getUUID())) return true;
        if (!hasPermission(viewer.getUUID(), PERM_SEE)) return false;
        int viewerWeight = getMaxRoleWeight(viewer.getUUID());
        int targetWeight = getMaxRoleWeight(target.getUUID());
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

    public void toggleVanish(ServerPlayer player, String lang) {
        setVanish(player, !isVanished(player.getUUID()), lang);
    }

    public void setVanish(ServerPlayer player, boolean enable, String lang) {
        if (enable && !isVanishFullySupported()) {
            AgentRuntime.LOG.warn("Vanish rejected for {}: mixins incomplete (TrackedEntity={}, ServerLevel={}, ItemEntity={})",
                    player.getScoreboardName(), trackedEntityMixinLoaded, serverLevelMixinLoaded, itemEntityMixinLoaded);
            ModText.send(player, ModText.parse("#f87171Vanish is disabled on this server version (level mixins incomplete)."));
            return;
        }
        UUID uuid = player.getUUID();
        if (enable) {
            vanished.add(uuid);
            if (server != null) {
                for (ServerPlayer other : server.getPlayerList().getPlayers()) {
                    if (other.equals(player)) continue;
                    if (!canSee(other, player)) {
                        ModText.send(other, ModText.translatable("multiplayer.player.left", player.getDisplayName()));
                    }
                }
            }
            ModText.send(player, AgentStrings.get(lang, "vanish_enabled"));
        } else {
            vanished.remove(uuid);
            if (server != null) {
                for (ServerPlayer other : server.getPlayerList().getPlayers()) {
                    if (other.equals(player)) continue;
                    if (!canSee(other, player)) {
                        ModText.send(other, ModText.translatable("multiplayer.player.joined", player.getDisplayName()));
                    }
                }
            }
            ModText.send(player, AgentStrings.get(lang, "vanish_disabled"));
        }
    }

    public void sendStaffChat(ServerPlayer vanishedPlayer, String rawMessage) {
        Component formatted = ModText.parse("#6b7280[#f87171VANISH#6b7280] #ffffff" + vanishedPlayer.getScoreboardName() + "#9ca3af: #ffffff" + rawMessage);
        ModText.send(vanishedPlayer, formatted);
        if (server != null) {
            for (ServerPlayer other : server.getPlayerList().getPlayers()) {
                if (other.equals(vanishedPlayer)) continue;
                if (hasPermission(other.getUUID(), PERM_STAFF) || canSee(other, vanishedPlayer)) {
                    ModText.send(other, formatted);
                }
            }
        }
    }

    public void sendVanishList(ServerPlayer sender) {
        if (!hasPermission(sender.getUUID(), PERM_SEE)) {
            ModText.send(sender, AgentStrings.get(null, "no_perm_view"));
            return;
        }
        StringBuilder sb = new StringBuilder("#9ca3afСкрытые модераторы (#ffffff").append(vanished.size()).append("#9ca3af): ");
        boolean first = true;
        if (server != null) {
            for (UUID vUuid : vanished) {
                ServerPlayer p = server.getPlayerList().getPlayer(vUuid);
                if (p != null) {
                    if (!first) sb.append("#9ca3af, ");
                    sb.append("#ffffff").append(p.getScoreboardName());
                    first = false;
                }
            }
        }
        if (first) sb.append("#4b5563(нет)");
        ModText.send(sender, sb.toString());
    }
}
