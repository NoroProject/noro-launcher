package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.Test;

class PlaceholderValuesTest {

    private static final UUID UUID_ONE = UUID.fromString("00000000-0000-0000-0000-000000000001");

    /** Старшая роль без иконки: в игре видна иконка следующей за ней. */
    private static final RoleInfo OWNER = new RoleInfo("owner", "Owner", "owner", "#ff0000", null, null, null, 100);

    private static final RoleInfo ADMIN = new RoleInfo("admin", "Админ", "admin", "#ff8c82", "★", null, null, 50);

    private static final RoleInfo VIP = new RoleInfo("vip", "VIP", "vip", "#00ff00", "✦", null, null, 10);

    private static PlayerProfile profile() {
        return new PlayerProfile(
                UUID_ONE,
                "Steve",
                false,
                true,
                false,
                null,
                List.of(),
                List.of(VIP, OWNER, ADMIN),
                "https://cdn/skin.png",
                null,
                List.of("owner", "admin", "vip"),
                List.of("noro.fly", "essentials.*"));
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
        // owner старше, но показывать ему нечего — префикс даёт admin.
        assertEquals("§x§f§f§8§c§8§2★§r", PlaceholderValues.resolve(profile(), "prefix"));
        assertEquals("★", PlaceholderValues.resolve(profile(), "prefix_plain"));
    }

    /**
     * Заданный префикс вытесняет иконку: иконка — глиф для таба, а префикс —
     * то, что владелец сервера написал сам.
     */
    @Test
    void ownPrefixWinsOverIcon() {
        RoleInfo staff = new RoleInfo("staff", "Staff", null, "#ff8c82", "★", "&8[&cSTAFF&8] ", " &7#1", 100);
        PlayerProfile profile = new PlayerProfile(
                UUID_ONE, "Steve", false, true, false, null, List.of(), List.of(staff), null, null,
                List.of(), List.of());

        assertEquals("§8[§cSTAFF§8] ", PlaceholderValues.resolve(profile, "prefix"));
        assertEquals("[STAFF] ", PlaceholderValues.resolve(profile, "prefix_plain"));
        assertEquals(" §7#1", PlaceholderValues.resolve(profile, "suffix"));
        assertEquals(" #1", PlaceholderValues.resolve(profile, "suffix_plain"));
        // Иконка при этом остаётся сама собой и доступна отдельным ключом.
        assertEquals("★", PlaceholderValues.resolve(profile, "role_icon"));
    }

    /** Суффикса нет — ключ отдаёт пустую строку, а не «null» посреди ника. */
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
        // Wildcard раскрывается так же, как при выдаче прав.
        assertEquals("true", PlaceholderValues.resolve(profile(), "permission_essentials.home"));
        assertEquals("false", PlaceholderValues.resolve(profile(), "permission_noro.ban"));
    }

    @Test
    void unknownKeyStaysUnanswered() {
        assertNull(PlaceholderValues.resolve(profile(), "nope"));
        // Голый префикс без хвоста — тоже не наш ключ.
        assertNull(PlaceholderValues.resolve(profile(), "has_role_"));
        assertNull(PlaceholderValues.resolve(null, "role"));
    }

    /**
     * Сервер без LuckPerms — у ролей нет и не будет `lp_group`, но иконка есть.
     * Раньше префикс был завязан на группу, и в таком раскладе он пропадал у
     * всех: в чате и в табе не появлялось ничего.
     */
    @Test
    void prefixDoesNotNeedALuckPermsGroup() {
        RoleInfo groupless = new RoleInfo("admin", "Админ", null, "#5865f2", "★", null, null, 100);
        PlayerProfile profile = new PlayerProfile(
                UUID_ONE, "Steve", false, true, false, null, List.of(), List.of(groupless), null, null,
                List.of(), List.of());

        assertEquals("§x§5§8§6§5§f§2★§r", PlaceholderValues.resolve(profile, "prefix"));
        assertEquals("★", PlaceholderValues.resolve(profile, "prefix_plain"));
        // roles_icons отдаёт иконки в цвете своих ролей, а не голыми символами.
        assertEquals("§x§5§8§6§5§f§2★§r", PlaceholderValues.resolve(profile, "roles_icons"));
    }

    @Test
    void missingDataBecomesEmptyString() {
        PlayerProfile bare = new PlayerProfile(
                UUID_ONE, "Steve", false, true, false, null, List.of(), List.of(), null, null,
                List.of(), List.of());
        assertEquals("", PlaceholderValues.resolve(bare, "prefix"));
        assertEquals("", PlaceholderValues.resolve(bare, "role"));
        assertEquals("", PlaceholderValues.resolve(bare, "cape_url"));
        assertEquals("", PlaceholderValues.resolve(bare, "group"));
        assertEquals("0", PlaceholderValues.resolve(bare, "permission_count"));
    }
}
