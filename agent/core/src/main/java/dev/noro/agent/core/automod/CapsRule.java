package dev.noro.agent.core.automod;

/**
 * Фильтр капса: доля заглавных букв в сообщении.
 */
public final class CapsRule {

    private CapsRule() {}

    public static boolean check(String text, FilterConfig config) {
        if (config == null || !config.enabled()) {
            return false;
        }

        int minLen = config.minLength() > 0 ? config.minLength() : 6;
        if (text.length() < minLen) {
            return false;
        }

        int upperCount = 0;
        int letterCount = 0;

        for (int i = 0; i < text.length(); i++) {
            char c = text.charAt(i);
            if (Character.isLetter(c)) {
                letterCount++;
                if (Character.isUpperCase(c)) {
                    upperCount++;
                }
            }
        }

        if (letterCount < minLen) {
            return false;
        }

        double threshold = config.threshold() > 0 ? config.threshold() : 0.6;
        return ((double) upperCount / letterCount) >= threshold;
    }
}
