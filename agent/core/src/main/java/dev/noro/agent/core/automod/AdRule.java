package dev.noro.agent.core.automod;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Фильтр рекламы: IP-адреса, домены, discord-инвайты с учётом белого списка.
 */
public final class AdRule {

    private static final Pattern IP_PATTERN = Pattern.compile("\\b\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\b");
    private static final Pattern IP_STRIPPED_PATTERN = Pattern.compile("\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}");
    private static final Pattern DISCORD_PATTERN = Pattern.compile("(discord\\.gg|discord\\.com/invite)/[a-zA-Z0-9]+");
    private static final Pattern DOMAIN_PATTERN = Pattern.compile("\\b[a-zA-Z0-9.-]+\\.(com|ru|net|org|gg|dev|store|io|me|online|xyz)\\b");

    private AdRule() {}

    public static boolean check(String text, FilterConfig config) {
        if (config == null || !config.enabled()) {
            return false;
        }

        String lowerRaw = text.toLowerCase(java.util.Locale.ROOT);
        String stripped = lowerRaw.replaceAll("[\\s_\\-]+", "");

        Matcher ipMatcher = IP_PATTERN.matcher(text);
        if (ipMatcher.find() || IP_STRIPPED_PATTERN.matcher(stripped).find()) {
            return true;
        }

        Matcher discordMatcher = DISCORD_PATTERN.matcher(lowerRaw);
        if (discordMatcher.find() || DISCORD_PATTERN.matcher(stripped).find()) {
            return true;
        }

        Matcher domainMatcher = DOMAIN_PATTERN.matcher(lowerRaw);
        while (domainMatcher.find()) {
            String domain = domainMatcher.group();
            boolean isWhitelisted = config.whitelist().stream().anyMatch(w -> domain.endsWith(w.toLowerCase(java.util.Locale.ROOT)));
            if (!isWhitelisted) {
                return true;
            }
        }

        return false;
    }
}
