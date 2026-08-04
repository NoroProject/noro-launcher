package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;

class RoleSyncDiffTest {

    @Test
    void addsMissingGroups() {
        RoleSync.Diff diff = RoleSync.diff(Set.of("player"), List.of("admin", "player"));
        assertEquals(Set.of("admin"), diff.toAdd());
        assertTrue(diff.toRemove().isEmpty());
    }

    @Test
    void removesGroupsMasterNoLongerLists() {
        // Ради этого случая всё и затевалось: роль сняли на сайте, и она обязана
        // исчезнуть в игре, а не остаться навсегда.
        RoleSync.Diff diff = RoleSync.diff(Set.of("admin", "player"), List.of("player"));
        assertEquals(Set.of("admin"), diff.toRemove());
        assertTrue(diff.toAdd().isEmpty());
    }

    @Test
    void doesNothingWhenAlreadyInSync() {
        RoleSync.Diff diff = RoleSync.diff(Set.of("admin", "player"), List.of("player", "admin"));
        assertTrue(diff.toAdd().isEmpty());
        assertTrue(diff.toRemove().isEmpty());
    }

    @Test
    void emptyTargetStripsEverythingManaged() {
        RoleSync.Diff diff = RoleSync.diff(Set.of("admin", "vip"), List.of());
        assertEquals(Set.of("admin", "vip"), diff.toRemove());
    }
}
