package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.Test;

class PlaceholderValuesTest {

    private static final UUID UUID_ONE = UUID.fromString("00000000-0000-0000-0000-000000000001");

    private static final RoleInfo OWNER = new RoleInfo("owner", "Owner", "owner", "#ff0000", null, null, null, 100);
    private static final RoleInfo ADMIN = new RoleInfo("admin", "Админ", "admin", "#ff8c82", "★", null, null, 50);
    private static final RoleInfo VIP = new RoleInfo("vip", "VIP", "vip", "#00ff00", "✦", null, null, 10);

    private static PlayerProfile profile() {
        return new PlayerProfile(
                UUID_ONE,
                "Steve",
                false,
                true,
                null,
                false,
                false,
                null,
                null,
                List.of(),
                List.of(VIP, OWNER, ADMIN),
                "https://cdn/skin.png",
                null,
                List.of("owner", "admin", "vip"),
                List.of("noro.fly", "essentials.*"),
                null,
                null,
                false);
    }

    @Test
    void takesTopRoleBySortOrder() {
        assertEquals("Owner", PlaceholderValues.resolve(profile(), "role"));
        assertEquals("owner", PlaceholderValues.resolve(profile(), "role_name"));
        assertEquals("100", PlaceholderValues.resolve(profile(), "role_sort"));
        assertEquals("3", PlaceholderValues.resolve(profile(), "role_count"));
    }

    @Test
    void prefixSkipsRolesWithoutDecoration() {
        assertEquals("§x§f§f§8§c§8§2★§r", PlaceholderValues.resolve(profile(), "prefix"));
        assertEquals("★", PlaceholderValues.resolve(profile(), "prefix_plain"));
    }

    @Test
    void ownPrefixWinsOverIcon() {
        RoleInfo staff = new RoleInfo("staff", "Staff", null, "#ff8c82", "★", "&8[&cSTAFF&8] ", " &7#1", 100);
        PlayerProfile profile = new PlayerProfile(
                UUID_ONE, "Steve", false, true, null, false, false, null, null, List.of(), List.of(staff), null, null,
                List.of(), List.of(), null, null, false);

        assertEquals("§8[§cSTAFF§8] ", PlaceholderValues.resolve(profile, "prefix"));
        assertEquals("[STAFF] ", PlaceholderValues.resolve(profile, "prefix_plain"));
        assertEquals(" §7#1", PlaceholderValues.resolve(profile, "suffix"));
        assertEquals(" #1", PlaceholderValues.resolve(profile, "suffix_plain"));
        assertEquals("★", PlaceholderValues.resolve(profile, "role_icon"));
    }

    @Test
    void missingSuffixIsEmpty() {
        assertEquals("", PlaceholderValues.resolve(profile(), "suffix"));
    }

    @Test
    void iconsGoInOrderOfImportance() {
        assertEquals("§x§f§f§8§c§8§2★§r§x§0§0§f§f§0§0✦§r", PlaceholderValues.resolve(profile(), "roles_icons"));
        assertEquals("Owner, Админ, VIP", PlaceholderValues.resolve(profile(), "roles"));
    }

    @Test
    void groupsKeepMasterOrder() {
        assertEquals("owner", PlaceholderValues.resolve(profile(), "group"));
        assertEquals("owner, admin, vip", PlaceholderValues.resolve(profile(), "groups"));
        assertEquals("3", PlaceholderValues.resolve(profile(), "group_count"));
    }

    @Test
    void answersMembershipAndPermissions() {
        assertEquals("true", PlaceholderValues.resolve(profile(), "has_role_admin"));
        assertEquals("false", PlaceholderValues.resolve(profile(), "has_role_mod"));
        assertEquals("true", PlaceholderValues.resolve(profile(), "has_group_vip"));
        assertEquals("true", PlaceholderValues.resolve(profile(), "permission_noro.fly"));
        assertEquals("true", PlaceholderValues.resolve(profile(), "permission_essentials.home"));
        assertEquals("false", PlaceholderValues.resolve(profile(), "permission_noro.ban"));
    }

    @Test
    void unknownKeyStaysUnanswered() {
        assertNull(PlaceholderValues.resolve(profile(), "nope"));
        assertNull(PlaceholderValues.resolve(profile(), "has_role_"));
        assertNull(PlaceholderValues.resolve(null, "role"));
    }

    @Test
    void prefixDoesNotNeedALuckPermsGroup() {
        RoleInfo groupless = new RoleInfo("admin", "Админ", null, "#5865f2", "★", null, null, 100);
        PlayerProfile profile = new PlayerProfile(
                UUID_ONE, "Steve", false, true, null, false, false, null, null, List.of(), List.of(groupless), null, null,
                List.of(), List.of(), null, null, false);

        assertEquals("§x§5§8§6§5§f§2★§r", PlaceholderValues.resolve(profile, "prefix"));
        assertEquals("★", PlaceholderValues.resolve(profile, "prefix_plain"));
        assertEquals("§x§5§8§6§5§f§2★§r", PlaceholderValues.resolve(profile, "roles_icons"));
    }

    @Test
    void missingDataBecomesEmptyString() {
        PlayerProfile bare = new PlayerProfile(
                UUID_ONE, "Steve", false, true, null, false, false, null, null, List.of(), List.of(), null, null,
                List.of(), List.of(), null, null, false);
        assertEquals("", PlaceholderValues.resolve(bare, "prefix"));
        assertEquals("", PlaceholderValues.resolve(bare, "role"));
        assertEquals("", PlaceholderValues.resolve(bare, "cape_url"));
        assertEquals("", PlaceholderValues.resolve(bare, "group"));
        assertEquals("0", PlaceholderValues.resolve(bare, "permission_count"));
    }
}
