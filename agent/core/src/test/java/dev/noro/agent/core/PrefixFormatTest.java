package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

import org.junit.jupiter.api.Test;

class PrefixFormatTest {

    @Test
    void buildsLegacyHexPrefix() {
        // Пример из спеки: роль «Админ», #ff8c82 и звезда.
        assertEquals("§x§f§f§8§c§8§2★§r", PrefixFormat.of("#ff8c82", "★"));
    }

    @Test
    void normalizesUppercaseHex() {
        assertEquals("§x§f§f§8§c§8§2★§r", PrefixFormat.of("#FF8C82", "★"));
    }

    @Test
    void noIconMeansNoPrefix() {
        assertNull(PrefixFormat.of("#ff8c82", null));
        assertNull(PrefixFormat.of("#ff8c82", "  "));
    }

    @Test
    void malformedColorFallsBackToPlainIcon() {
        // Цвет необязателен и мог прийти мусором — иконку это терять не повод.
        assertEquals("★§r", PrefixFormat.of(null, "★"));
        assertEquals("★§r", PrefixFormat.of("ff8c82", "★"));
        assertEquals("★§r", PrefixFormat.of("#xyzxyz", "★"));
    }
}
