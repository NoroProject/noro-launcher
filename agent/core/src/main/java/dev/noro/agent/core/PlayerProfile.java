package dev.noro.agent.core;

import java.util.List;
import java.util.UUID;

/**
 * Ответ {@code GET /api/agent/players/{mc_uuid}}.
 */
public record PlayerProfile(
        UUID uuid,
        String username,
        boolean banned,
        boolean allowed,
        String denialReason,
        boolean maintenanceBypass,
        boolean muted,
        PunishmentInfo activeMute,
        PunishmentInfo activeBan,
        List<PunishmentInfo> pendingWarns,
        List<RoleInfo> roles,
        String skinUrl,
        String capeUrl,
        List<String> lpGroups,
        List<String> permissions,
        String locale,
        FreezeInfo freezeInfo,
        boolean vanishOnJoin) {

    public PlayerProfile {
        roles = roles == null ? List.of() : List.copyOf(roles);
        lpGroups = lpGroups == null ? List.of() : List.copyOf(lpGroups);
        permissions = permissions == null ? List.of() : List.copyOf(permissions);
        pendingWarns = pendingWarns == null ? List.of() : List.copyOf(pendingWarns);
    }

    public boolean frozen() {
        return freezeInfo != null;
    }

    public boolean hasPermission(String perm) {
        if ("noro.server.maintenance.bypass".equals(perm)) {
            return maintenanceBypass;
        }
        if (permissions == null || permissions.isEmpty()) {
            return false;
        }
        return permissions.contains(perm) || permissions.contains("*");
    }
}
