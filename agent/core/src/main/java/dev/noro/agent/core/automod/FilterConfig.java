package dev.noro.agent.core.automod;

import java.util.List;

/**
 * Конфигурация отдельного фильтра чата.
 */
public record FilterConfig(
        String filterType,
        /** deny / escalate / punish / shadow */
        String mode,
        boolean enabled,
        String ruleCode,
        List<String> whitelist,
        List<String> words,
        double threshold,
        int minLength,
        int maxMessages,
        int windowSecs) {

    public FilterConfig {
        whitelist = whitelist == null ? List.of() : List.copyOf(whitelist);
        words = words == null ? List.of() : List.copyOf(words);
        mode = mode == null ? "deny" : mode;
    }
}
