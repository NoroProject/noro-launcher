package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.util.List;
import java.util.Optional;
import java.util.UUID;
import org.junit.jupiter.api.Test;

class MasterClientTest {

    private static final UUID PLAYER = UUID.fromString("95bf010b-8e9f-55d5-8f1c-766c624ab7e0");

    @Test
    void parsesPlayerResponse() throws Exception {
        try (StubMaster master = StubMaster.start(200, StubMaster.PLAYER_JSON)) {
            Optional<PlayerProfile> found = new MasterClient(master.config()).player(PLAYER);

            assertTrue(found.isPresent());
            PlayerProfile profile = found.get();
            assertEquals(PLAYER, profile.uuid());
            assertEquals("dalynkaa", profile.username());
            assertFalse(profile.banned());
            assertTrue(profile.allowed());
            assertEquals(List.of("admin", "player"), profile.lpGroups());
            assertEquals("http://example/api/textures/default-skin", profile.skinUrl());
            assertNull(profile.capeUrl());

            RoleInfo role = profile.roles().get(0);
            assertEquals("admin", role.lpGroup());
            assertEquals("Админ", role.displayName());
            assertEquals(100, role.sortOrder());
            assertTrue(role.hasPrefix());
        }
    }

    @Test
    void parsesPermissionsIncludingPrefixNodes() throws Exception {
        try (StubMaster master = StubMaster.start(200, StubMaster.PLAYER_JSON)) {
            PlayerProfile profile = new MasterClient(master.config()).player(PLAYER).orElseThrow();
            PermissionSet perms = PermissionSet.of(profile.permissions());

            assertTrue(perms.has("noro.admin.users"));
            assertTrue(perms.has("servercore.command.settings"));
            assertFalse(perms.has("servercore.command.other"));
            // Префикс приезжает узлом права: моды читают меты именно оттуда.
            assertTrue(profile.permissions().contains("prefix.100.§x§f§f§8§c§8§2★§r"));
        }
    }

    @Test
    void missingPermissionsFieldIsEmptyNotNull() throws Exception {
        // Старый мастер поля ещё не отдаёт — агент обязан это пережить.
        String legacy = StubMaster.PLAYER_JSON.replaceAll("(?s),\\s*\"permissions\".*?\\]", "");
        try (StubMaster master = StubMaster.start(200, legacy)) {
            PlayerProfile profile = new MasterClient(master.config()).player(PLAYER).orElseThrow();
            assertEquals(List.of(), profile.permissions());
        }
    }

    @Test
    void reportsNodeCatalog() throws Exception {
        try (StubMaster master = StubMaster.start(200, "{\"ok\":true,\"accepted\":2}")) {
            new MasterClient(master.config())
                    .reportNodes(List.of("servercore.command.settings", "noro.agent.test"));

            assertEquals("/api/agent/nodes", master.lastPath);
            assertEquals("Bearer " + StubMaster.SECRET, master.lastAuthHeader);
            assertTrue(master.lastBody.contains("\"servercore.command.settings\""), master.lastBody);
            assertTrue(master.lastBody.startsWith("{\"nodes\":["), master.lastBody);
        }
    }

    @Test
    void sendsBearerSecretAndUuidPath() throws Exception {
        try (StubMaster master = StubMaster.start(200, StubMaster.PLAYER_JSON)) {
            new MasterClient(master.config()).player(PLAYER);

            assertEquals("Bearer " + StubMaster.SECRET, master.lastAuthHeader);
            // server_id мастер берёт из секрета — в пути только UUID игрока.
            assertEquals("/api/agent/players/" + PLAYER, master.lastPath);
        }
    }

    @Test
    void unknownPlayerIsEmptyNotError() throws Exception {
        try (StubMaster master = StubMaster.start(404, "player not found")) {
            assertTrue(new MasterClient(master.config()).player(PLAYER).isEmpty());
        }
    }

    @Test
    void rejectedSecretExplainsTheFix() throws Exception {
        try (StubMaster master = StubMaster.start(401, "unauthorized")) {
            IOException error =
                    assertThrows(IOException.class, () -> new MasterClient(master.config()).player(PLAYER));
            assertTrue(error.getMessage().contains("Game servers"), error.getMessage());
        }
    }

    @Test
    void heartbeatSendsSnakeCaseBody() throws Exception {
        try (StubMaster master = StubMaster.start(200, "{\"ok\":true}")) {
            new MasterClient(master.config()).heartbeat(12, 60, "Paper 1.21.1");

            assertEquals("/api/agent/heartbeat", master.lastPath);
            assertTrue(master.lastBody.contains("\"max_players\":60"), master.lastBody);
            assertTrue(master.lastBody.contains("\"online\":12"), master.lastBody);
            assertTrue(master.lastBody.contains("\"version\":\"Paper 1.21.1\""), master.lastBody);
        }
    }
}
