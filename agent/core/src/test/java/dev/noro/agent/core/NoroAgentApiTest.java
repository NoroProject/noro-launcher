package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;

class NoroAgentApiTest {

    private static final UUID PLAYER = UUID.fromString("00000000-0000-0000-0000-000000000002");

    /** Старшая роль без иконки: префикс должен взяться у следующей. */
    private static final RoleInfo OWNER = new RoleInfo("owner", "Owner", "owner", "#ff0000", null, null, null, 100);

    private static final RoleInfo ADMIN = new RoleInfo("admin", "Админ", "admin", "#ff8c82", "★", null, null, 50);

    @AfterEach
    void clear() {
        // Кэш здесь статический, и тесты не должны видеть чужие профили.
        NoroAgentApi.cache().forget(PLAYER);
    }

    private static void online() {
        NoroAgentApi.cache()
                .remember(PLAYER, new PlayerProfile(
                        PLAYER, "Steve", false, true, false, null, List.of(), List.of(OWNER, ADMIN), null, null,
                        List.of(), List.of()));
    }

    @Test
    void givesHexColorAndIconSeparately() {
        online();
        RoleInfo role = NoroAgentApi.prefixRole(PLAYER);
        // Ровно то, что нужно моду с поддержкой hex: цвет и иконка не слиты.
        assertEquals("#ff8c82", role.color());
        assertEquals("★", role.icon());
        assertEquals("Админ", role.displayName());
    }

    @Test
    void topRoleIgnoresIcons() {
        online();
        assertEquals("owner", NoroAgentApi.topRole(PLAYER).name());
        assertEquals("admin", NoroAgentApi.prefixRole(PLAYER).name());
    }

    @Test
    void stringApiMatchesPlaceholders() {
        online();
        assertEquals("#ff0000", NoroAgentApi.value(PLAYER, "role_color"));
        assertEquals("§x§f§f§8§c§8§2★§r", NoroAgentApi.value(PLAYER, "prefix"));
        assertEquals("true", NoroAgentApi.value(PLAYER, "has_role_admin"));
    }

    @Test
    void offlinePlayerHasNothing() {
        assertNull(NoroAgentApi.profile(PLAYER));
        assertNull(NoroAgentApi.prefixRole(PLAYER));
        assertNull(NoroAgentApi.topRole(PLAYER));
        assertNull(NoroAgentApi.value(PLAYER, "role_color"));
    }
}
