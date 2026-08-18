package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.Test;

class FreezeCommandTest {

    private static class TestSender implements CommandSender {
        final List<String> replies = new ArrayList<>();

        @Override public UUID uuid() { return UUID.randomUUID(); }
        @Override public String name() { return "Steve"; }
        @Override public boolean console() { return false; }
        @Override public boolean has(String p) { return true; }
        @Override public void reply(String m) { replies.add(m); }
    }

    @Test
    void testFreezePermissionDenied() {
        TestSender sender = new TestSender() {
            @Override public boolean has(String p) { return false; }
        };
        FreezeCommand freezeCmd = new FreezeCommand(null, null, null);
        freezeCmd.freeze(sender, new String[]{"Steve"}, "en");
        assertFalse(sender.replies.isEmpty());
        assertTrue(sender.replies.get(0).contains("cannot freeze") || sender.replies.get(0).contains("no_perm_freeze"));
    }
}
