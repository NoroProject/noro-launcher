package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

/**
 * Те же случаи, что и в {@code wildcard_matching} на стороне мастера: если эти
 * два теста разойдутся, разойдутся и ответы сайта с ответами игры.
 */
class PermissionSetTest {

    @Test
    void wildcardMatchingRepeatsMaster() {
        assertTrue(PermissionSet.matches("*", "anything.here"));
        assertTrue(PermissionSet.matches("noro.server.*", "noro.server.hitech"));
        assertTrue(PermissionSet.matches("noro.server.*", "noro.server.hitech.join"));
        assertTrue(PermissionSet.matches("noro.server.hitech.join", "noro.server.hitech.join"));
        assertFalse(PermissionSet.matches("noro.server.*", "noro.admin.users"));
        assertFalse(PermissionSet.matches("noro.server.hitech", "noro.server.hitech2"));
    }

    @Test
    void wildcardDoesNotLeakIntoSiblingBranch() {
        // `foo.*` не должен цеплять `foobar`: точка — часть шаблона, а не украшение.
        assertFalse(PermissionSet.matches("foo.*", "foobar"));
        assertFalse(PermissionSet.matches("foo.*", "foobar.baz"));
    }

    @Test
    void starInTheMiddleIsNotAWildcard() {
        // Мастер поддерживает только суффиксный `.*`, и агент не должен добавлять
        // от себя более широкое правило — иначе он выдаст больше, чем сайт.
        assertFalse(PermissionSet.matches("noro.*.join", "noro.server.join"));
    }

    @Test
    void setAnswersThroughAnyPattern() {
        PermissionSet perms = PermissionSet.of(List.of("servercore.command.settings", "noro.server.*"));

        assertTrue(perms.has("servercore.command.settings"));
        assertTrue(perms.has("noro.server.hitech.join"));
        assertFalse(perms.has("servercore.command.other"));
        assertFalse(perms.has("noro.admin.users"));
    }

    @Test
    void superadminAnswersEverything() {
        assertTrue(PermissionSet.of(List.of("*")).has("anything.at.all"));
    }

    @Test
    void emptySetAnswersNothing() {
        assertFalse(PermissionSet.empty().has("servercore.command.settings"));
        assertFalse(PermissionSet.of(null).has("*"));
    }
}
