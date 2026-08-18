package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.Test;

class CheckCommandTest {

    private static class TestSender implements CommandSender {
        final List<String> replies = new ArrayList<>();

        @Override public UUID uuid() { return UUID.randomUUID(); }
        @Override public String name() { return "Steve"; }
        @Override public boolean console() { return false; }
        @Override public boolean has(String p) { return true; }
        @Override public void reply(String m) { replies.add(m); }
    }

    @Test
    void testCheckPermissionRequired() {
        TestSender sender = new TestSender() {
            @Override public boolean has(String p) { return false; }
        };
        CheckCommand check = new CheckCommand(null, null);
        check.check(sender, new String[]{"Steve"}, "en");
        assertFalse(sender.replies.isEmpty());
        assertTrue(sender.replies.get(0).contains("cannot see") || sender.replies.get(0).contains("no_perm_view"));
    }

    @Test
    void testCheckUsageWhenNoArgs() {
        TestSender sender = new TestSender();
        CheckCommand check = new CheckCommand(null, null);
        check.check(sender, new String[0], "en");
        assertFalse(sender.replies.isEmpty());
    }
}
