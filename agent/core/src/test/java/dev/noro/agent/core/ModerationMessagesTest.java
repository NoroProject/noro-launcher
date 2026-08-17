package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.time.Instant;
import java.util.UUID;
import org.junit.jupiter.api.Test;

/** Тексты, которые видит наказанный: подстановка, цвета и выбор шаблона. */
class ModerationMessagesTest {

    private static final UUID CASE = UUID.fromString("1a2b3c4d-0000-0000-0000-000000000000");

    private static PunishmentInfo ban(Instant expires) {
        return new PunishmentInfo(
                CASE, "ban", "cheating", "admin", Instant.parse("2026-08-17T05:00:00Z"), expires, null, "game.cheats");
    }

    @Test
    void fillsEveryPlaceholder() {
        String text = MessageRender.render(
                "{player} {kind} by {actor}: {reason} [{rule}] case {id} expires {expires}",
                ban(null),
                "Steve");

        assertEquals("Steve banned by admin: cheating [game.cheats] case 1a2b3c4d expires never", text);
    }

    @Test
    void turnsAmpersandIntoSectionSign() {
        String text = MessageRender.render("&cBanned:&r {reason}", ban(null), "Steve");

        assertEquals("§cBanned:§r cheating", text);
        // Одинокий амперсанд — это просто амперсанд, а не начало цвета.
        assertEquals("tom & jerry", MessageRender.render("tom & jerry", ban(null), "Steve"));
    }

    @Test
    void picksTemplateByKindAndTerm() {
        MessageTemplates templates = MessageTemplates.defaults();

        assertEquals(templates.banPermanent(), templates.screen(ban(null)));
        assertEquals(templates.banTemporary(), templates.screen(ban(Instant.now().plusSeconds(3600))));
    }

    /** Мастер мог не прислать поле — экран бана всё равно не должен быть пустым. */
    @Test
    void completesMissingTemplatesWithBuiltIn() {
        MessageTemplates partial =
                new MessageTemplates(null, "", null, null, null, null, null, "", null).complete();

        assertEquals(MessageTemplates.defaults().banPermanent(), partial.banPermanent());
        assertEquals(MessageTemplates.defaults().banTemporary(), partial.banTemporary());
        // Молчаливое объявление — законный выбор, и пустая строка сохраняется.
        assertEquals("", partial.broadcast());
    }

    @Test
    void knowsWhichCommandsSpeak() {
        assertTrue(ChatCommands.speaks("/msg Steve hi"));
        assertTrue(ChatCommands.speaks("me waves"));
        // Плагины регистрируют команды и с префиксом своего имени.
        assertTrue(ChatCommands.speaks("/essentials:msg Steve hi"));
        assertFalse(ChatCommands.speaks("/home"));
        assertFalse(ChatCommands.speaks(null));
    }
}
