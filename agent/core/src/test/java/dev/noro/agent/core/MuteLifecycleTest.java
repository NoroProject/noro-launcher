package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.time.Instant;
import java.util.UUID;
import org.junit.jupiter.api.Test;

/**
 * Жизнь мута от ответа мастера до отпускания чата.
 *
 * <p>Проверяется то, из-за чего игрок остаётся молчать после срока: срок,
 * потерянный при разборе JSON, читается как «навсегда», и на сервере это выглядит
 * как заглючивший мут, а не как ошибка парсинга.
 */
class MuteLifecycleTest {

    private static final UUID PLAYER = UUID.fromString("95bf010b-8e9f-55d5-8f1c-766c624ab7e0");

    /** Профиль с действующим мутом — в том виде, в каком его отдаёт мастер. */
    private static String profileJson(String expiresAt) {
        return """
                {
                  "uuid": "95bf010b-8e9f-55d5-8f1c-766c624ab7e0",
                  "username": "dalynkaa",
                  "banned": false,
                  "muted": true,
                  "active_mute": {
                    "id": "266be5b2-143c-4753-a1c2-7b6ac799ee4a",
                    "kind": "mute",
                    "reason": "spam",
                    "actor_label": "admin",
                    "created_at": "2026-08-17T08:00:00.123456Z",
                    "expires_at": %s,
                    "rule_code": "1.1"
                  },
                  "pending_warns": [],
                  "allowed": true,
                  "roles": [],
                  "skin_url": "http://example/skin",
                  "cape_url": null,
                  "lp_groups": [],
                  "permissions": []
                }
                """
                .formatted(expiresAt);
    }

    @Test
    void keepsMuteExpiryFromMaster() throws Exception {
        String future = "\"" + Instant.now().plusSeconds(600) + "\"";
        try (StubMaster master = StubMaster.start(200, profileJson(future))) {
            PlayerProfile profile = new MasterClient(master.config()).player(PLAYER).orElseThrow();

            PunishmentInfo mute = profile.activeMute();
            assertNotNull(mute, "мут из профиля обязан доехать до агента");
            assertEquals("mute", mute.kind());
            assertEquals("spam", mute.reason());
            // Главное: срок не потерян. Потерянный срок делает мут вечным.
            assertNotNull(mute.expiresAt(), "срок мута потерян при разборе JSON");
            assertTrue(mute.active());
        }
    }

    /** Истёкший мут отпускает чат сам: события о конце срока от мастера не будет. */
    @Test
    void expiredMuteReleasesChatWithoutMaster() throws Exception {
        String past = "\"" + Instant.now().minusSeconds(60) + "\"";
        try (StubMaster master = StubMaster.start(200, profileJson(past))) {
            PlayerProfile profile = new MasterClient(master.config()).player(PLAYER).orElseThrow();

            MuteRegistry mutes = new MuteRegistry();
            mutes.remember(PLAYER, profile.activeMute());

            assertNull(mutes.active(PLAYER), "истёкший мут обязан сниматься сам");
        }
    }

    /** Снятие мута с мастера доезжает кадром живого канала и открывает чат. */
    @Test
    void revokedFrameReleasesChat() {
        MuteRegistry mutes = new MuteRegistry();
        PunishmentInfo mute = new PunishmentInfo(
                UUID.randomUUID(), "mute", "spam", "admin", Instant.now(), null, null, null);
        mutes.remember(PLAYER, mute);
        assertNotNull(mutes.active(PLAYER));

        mutes.forget(PLAYER);
        assertNull(mutes.active(PLAYER));
    }
}
