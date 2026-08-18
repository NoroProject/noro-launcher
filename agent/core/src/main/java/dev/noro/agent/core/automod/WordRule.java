package dev.noro.agent.core.automod;

/**
 * Фильтр мата / запрещённых слов по словарю из админки с использованием нормализации.
 */
public final class WordRule {

    private WordRule() {}

    public static boolean check(String text, FilterConfig config) {
        if (config == null || !config.enabled() || config.words().isEmpty()) {
            return false;
        }

        String normalized = TextNormalize.normalize(text);
        String lowerRaw = text.toLowerCase(java.util.Locale.ROOT);

        for (String word : config.words()) {
            if (word.isBlank()) continue;
            String wLower = word.toLowerCase(java.util.Locale.ROOT);
            String wNorm = TextNormalize.normalize(word);

            if (!wNorm.isEmpty() && normalized.contains(wNorm)) {
                return true;
            }
            if (lowerRaw.contains(wLower)) {
                return true;
            }
        }

        return false;
    }
}
