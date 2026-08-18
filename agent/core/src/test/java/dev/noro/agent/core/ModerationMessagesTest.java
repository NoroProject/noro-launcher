package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.time.Instant;
import java.util.UUID;
import org.junit.jupiter.api.Test;

/** Тексты, которые видит наказанный: подстановка, выбор шаблона, разметка. */
class ModerationMessagesTest {

    private static final UUID CASE = UUID.fromString("1a2b3c4d-0000-0000-0000-000000000000");

    private static PunishmentInfo ban(Instant expires) {
        return new PunishmentInfo(
                CASE, "ban", "cheating", "admin", Instant.parse("2026-08-17T05:00:00Z"), expires, null, "1.1");
    }

    @Test
    void fillsEveryPlaceholder() {
        String text = MessageRender.render(
                "{player} {kind} by {actor}: {reason} [{rule}] {rule_title} case {id}",
                ban(null),
                "Steve",
                "Respect other players",
                null);

        assertEquals("Steve banned by admin: cheating [1.1] Respect other players case 1a2b3c4d", text);
    }

    /**
     * Ссылку на свод собирает агент, но только когда мастер прислал адрес:
     * угаданный домен в бане живёт годами и ведёт в никуда.
     */
    @Test
    void linksRuleOnlyWithMasterUrl() {
        assertEquals(
                "[1.1](https://noro.example/rules#rule-1.1)",
                MessageRender.render("{rule_link}", ban(null), "Steve", null, "https://noro.example/rules"));
        assertEquals("1.1", MessageRender.render("{rule_link}", ban(null), "Steve", null, ""));
    }

    /** Шаблон подставляется дословно: агент в него ничего не дописывает. */
    @Test
    void doesNotAppendAnythingToTemplate() {
        assertEquals("banned", MessageRender.render("{kind}", ban(null), "Steve", "title", "https://x/rules"));
    }

    @Test
    void picksTemplateByKindAndTerm() {
        MessageTemplates templates = MessageTemplates.defaults();

        assertEquals(templates.banPermanent(), templates.screen(ban(null)));
        assertEquals(templates.banTemporary(), templates.screen(ban(Instant.now().plusSeconds(3600))));
    }

    /** Над хотбаром — своя короткая строка, а не многострочный экран бана. */
    @Test
    void actionbarHasItsOwnText() {
        MessageTemplates templates = MessageTemplates.defaults();
        PunishmentInfo mute = new PunishmentInfo(
                CASE, "mute", "spam", "admin", Instant.now(), Instant.now().plusSeconds(600), null, null);

        assertEquals(templates.muteActionbarTemporary(), templates.actionbar(mute));
        assertFalse(templates.actionbar(mute).contains("\n"), "в actionbar одна строка");
    }

    /** Мастер мог не прислать поле — экран бана всё равно не должен быть пустым. */
    @Test
    void completesMissingTemplatesWithBuiltIn() {
        MessageTemplates partial = new MessageTemplates(
                null, "", null, null, null, null, null, null, null, null, "", null, null, null, null, null,
                null);
        MessageTemplates complete = partial.complete();

        assertEquals(MessageTemplates.defaults().banPermanent(), complete.banPermanent());
        assertEquals(MessageTemplates.defaults().muteActionbarTemporary(), complete.muteActionbarTemporary());
        // Молчаливое объявление — законный выбор, и пустая строка сохраняется.
        assertEquals("", complete.broadcast());
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
