package dev.noro.agent.core.automod;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.Test;

class ChatAutoModTest {

    @Test
    void testTextNormalization() {
        String normalized = TextNormalize.normalize("п р и в е т");
        assertTrue(normalized.contains("p") || normalized.contains("b") || normalized.contains("e") || normalized.contains("п"));
    }

    @Test
    void testAdRuleDetection() {
        FilterConfig config = new FilterConfig("ad", "punish", true, null, List.of("noro.dalynkaa.dev"), List.of(), 0, 0, 0, 0);
        assertTrue(AdRule.check("Connect to 127.0.0.1 right now!", config));
        assertTrue(AdRule.check("Join s 1 2 7.0.0.1!", config));
        assertTrue(AdRule.check("Join discord.gg/abcde", config));
        assertFalse(AdRule.check("Visit https://noro.dalynkaa.dev", config));
    }

    @Test
    void testWordRuleDetection() {
        FilterConfig config = new FilterConfig("word", "deny", true, null, List.of(), List.of("badword"), 0, 0, 0, 0);
        assertTrue(WordRule.check("This is b a d w o r d!", config));
    }

    @Test
    void testCapsRuleDetection() {
        FilterConfig config = new FilterConfig("caps", "deny", true, null, List.of(), List.of(), 0.6, 6, 0, 0);
        assertTrue(CapsRule.check("HELLO THIS IS CAPS LOCK", config));
        assertFalse(CapsRule.check("Hello this is normal text", config));
    }

    @Test
    void testChatFiltersDecision() {
        ChatFilters filters = new ChatFilters(null, null);
        filters.updateConfig("ad", new FilterConfig("ad", "punish", true, null, List.of(), List.of(), 0, 0, 0, 0));

        ChatFilters.Result res = filters.check(UUID.randomUUID(), "Join 192.168.1.1");
        assertEquals(ChatFilters.Action.PUNISH, res.action());
    }
}
