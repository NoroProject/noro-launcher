package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.Test;

/** Кому плашка, а кому прежний текстовый префикс. */
class PrefixServiceTest {

    private static final UUID VIEWER = UUID.randomUUID();

    private static RoleInfo admin() {
        return new RoleInfo("admin", "Админ", "admin", "#ff8c82", "★", null, null, 100);
    }

    private static PrefixService withPack() {
        PrefixService service = new PrefixService();
        service.pack(new PrefixPack(
                "http://master/files/abc", "abc", List.of(new PrefixPack.Glyph("admin", ""))));
        return service;
    }

    /**
     * Плашка показывается всем, независимо от зрителя.
     *
     * <p>Разное разным зрителям потребовало бы пересобирать текст сообщения на
     * каждого получателя, а он собирается один раз до рассылки. Пак едет вместе
     * со сборкой лаунчера, поэтому без него остаётся только зашедший чужим
     * клиентом.
     */
    @Test
    void showsTheGlyphToEveryone() {
        PrefixService service = withPack();

        assertEquals("\uE000", service.glyph(VIEWER, admin()));
        assertEquals("\uE000", service.glyph(UUID.randomUUID(), admin()));
        assertEquals("\uE000", service.glyph(null, admin()));
    }

    /** Роль без плашки остаётся без неё. */
    @Test
    void leavesRolesWithoutABadgeAlone() {
        PrefixService service = withPack();
        RoleInfo player = new RoleInfo("player", "Игрок", null, "#9CA3AF", null, null, null, 0);

        assertNull(service.glyph(VIEWER, player));
    }

    /** Пустой пак игрокам не выдаётся: выдавать нечего. */
    @Test
    void refusesToHandOutAnEmptyPack() {
        assertTrue(!new PrefixService().usable());
        assertTrue(withPack().usable());
    }

}
