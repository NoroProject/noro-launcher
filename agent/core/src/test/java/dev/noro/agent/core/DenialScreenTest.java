package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.time.Instant;
import java.util.UUID;
import org.junit.jupiter.api.Test;

class DenialScreenTest {

    private static final UUID CASE = UUID.fromString("7f3a0c1e-0000-4000-8000-000000000001");

    private static PunishmentInfo ban(String kind, Instant expires) {
        return new PunishmentInfo(CASE, kind, "griefing", "Admin", Instant.now(), expires, null, "3.1");
    }

    /**
     * То, ради чего п.13 и делался: забаненный вчера должен увидеть экран из
     * админки, а не литерал из кода.
     */
    @Test
    void banScreenComesFromTemplates() {
        AccessGate.Denial denial =
                new AccessGate.Denial(AccessGate.Reason.NETWORK_BAN, ban("ban", null), "Steve");

        String screen = DenialScreen.text(denial, MessageTemplates.defaults(), null);

        assertTrue(screen.contains("griefing"), screen);
        assertTrue(screen.contains("Admin"), screen);
        assertTrue(screen.contains(CASE.toString().substring(0, 8)), "номер дела нужен для апелляции");
        assertFalse(screen.contains("{reason}"), "подстановки не должны доезжать сырыми");
    }

    /** Бан со сроком и вечный — разные шаблоны, и срок обязан быть виден. */
    @Test
    void temporaryBanShowsItsEnd() {
        AccessGate.Denial denial = new AccessGate.Denial(
                AccessGate.Reason.NETWORK_BAN, ban("ban", Instant.now().plusSeconds(3600)), "Steve");

        String screen = DenialScreen.text(denial, MessageTemplates.defaults(), null);

        assertTrue(screen.contains("Expires"), screen);
    }

    /** Бан сборки не должен читаться как бан сети: это разные наказания. */
    @Test
    void serverBanUsesItsOwnTemplate() {
        AccessGate.Denial denial = new AccessGate.Denial(
                AccessGate.Reason.SERVER_BAN, ban("server_ban", null), "Steve");

        assertEquals(
                MessageRender.render(
                        MessageTemplates.defaults().serverBanPermanent(),
                        ban("server_ban", null),
                        "Steve",
                        null,
                        ""),
                DenialScreen.text(denial, MessageTemplates.defaults(), null));
    }

    /** Отказ без наказания подстановок не имеет — шаблон уходит как есть. */
    @Test
    void noAccountHasItsOwnScreen() {
        AccessGate.Denial denial = new AccessGate.Denial(AccessGate.Reason.NO_ACCOUNT, null, null);

        assertEquals(
                MessageTemplates.defaults().noAccount(),
                DenialScreen.text(denial, MessageTemplates.defaults(), null));
    }

    /**
     * Мастер недоступен — единственный случай, где текст свой: шаблоны могли не
     * доехать, и обещать набранный в админке экран нечем.
     */
    @Test
    void masterDownDoesNotPretendToHaveATemplate() {
        AccessGate.Denial denial = new AccessGate.Denial(AccessGate.Reason.MASTER_DOWN, null, null);

        assertTrue(DenialScreen.text(denial, MessageTemplates.defaults(), null).contains("unavailable"));
    }
}
