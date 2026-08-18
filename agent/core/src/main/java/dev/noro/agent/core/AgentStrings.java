package dev.noro.agent.core;

import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Чтение встроенных переводов агента из файлов ресурсов {@code /lang/{lang}.json}.
 */
public final class AgentStrings {

    private static final Gson GSON = new Gson();
    private static final Map<String, Map<String, String>> CACHE = new ConcurrentHashMap<>();

    private AgentStrings() {}

    public static String get(String lang, String key, Object... args) {
        String cleanLang = lang == null || lang.isBlank()
                ? "en"
                : lang.split("[-_]")[0].toLowerCase();
        Map<String, String> map = CACHE.computeIfAbsent(cleanLang, AgentStrings::loadLang);
        String template = map.get(key);
        if (template == null && !"en".equals(cleanLang)) {
            template = CACHE.computeIfAbsent("en", AgentStrings::loadLang).get(key);
        }
        if (template == null) {
            template = key;
        }
        for (int i = 0; i < args.length; i++) {
            template = template.replace("{" + i + "}", String.valueOf(args[i]));
        }
        return template;
    }

    private static Map<String, String> loadLang(String lang) {
        String resourcePath = "/lang/" + lang + ".json";
        try (InputStream is = AgentStrings.class.getResourceAsStream(resourcePath)) {
            if (is == null) {
                return Map.of();
            }
            try (InputStreamReader reader = new InputStreamReader(is, StandardCharsets.UTF_8)) {
                Map<String, String> loaded = GSON.fromJson(reader, new TypeToken<Map<String, String>>() {}.getType());
                return loaded == null ? Map.of() : Map.copyOf(loaded);
            }
        } catch (Exception e) {
            return Map.of();
        }
    }
}
