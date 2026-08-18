package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertFalse;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.Test;

class RuleCommandsTest {

    private static class TestSender implements CommandSender {
        final List<String> replies = new ArrayList<>();

        @Override public UUID uuid() { return null; }
        @Override public String name() { return "Console"; }
        @Override public boolean has(String p) { return true; }
        @Override public void reply(String m) { replies.add(m); }
    }

    @Test
    void testRulesListHandled() {
        TestSender sender = new TestSender();
        RuleCatalog catalog = new RuleCatalog(null);
        RuleCommands rulesCmd = new RuleCommands(catalog);
        rulesCmd.rules(sender, new String[0], "en");
        assertFalse(sender.replies.isEmpty());
    }
}
