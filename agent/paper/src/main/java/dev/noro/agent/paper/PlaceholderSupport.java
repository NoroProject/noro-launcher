package dev.noro.agent.paper;

import dev.noro.agent.core.ProfileCache;
import org.bukkit.plugin.Plugin;
import org.slf4j.Logger;

/**
 * Единственная дверь к PlaceholderAPI. За ней его может не быть.
 *
 * <p>Разнесение на два класса здесь не для красоты: типы PlaceholderAPI
 * упоминаются в {@link PaperPlaceholders}, и загрузка того класса без плагина
 * падает. Проверка наличия обязана жить там, где этих типов нет вовсе, — тем же
 * приёмом, что и {@code LuckPermsSupport} в core.
 */
final class PlaceholderSupport {

    private PlaceholderSupport() {}

    static void register(Plugin plugin, ProfileCache profiles, Logger log) {
        if (plugin.getServer().getPluginManager().getPlugin("PlaceholderAPI") == null) {
            log.info("PlaceholderAPI not found — %noro_…% placeholders are off");
            return;
        }
        try {
            Holder.register(plugin, profiles);
            log.info("Registered %noro_…% placeholders");
        } catch (Throwable e) {
            // Плейсхолдеры — украшение таба и чата, а не условие работы сервера.
            log.warn("PlaceholderAPI present but unusable, placeholders are off: {}", e.toString());
        }
    }

    /** Грузится только после того, как PlaceholderAPI подтверждён. */
    private static final class Holder {
        static void register(Plugin plugin, ProfileCache profiles) {
            new PaperPlaceholders(plugin, profiles).register();
        }
    }
}
