package dev.noro.agent.mod;

import dev.noro.agent.core.ProfileCache;

/**
 * Единственная дверь к Text Placeholder API. За ней его может не быть.
 *
 * <p>Проверка наличия обязана жить в классе, где типов Text Placeholder API нет
 * вовсе: загрузка {@link PlaceholderBridge} без установленного мода падает с
 * NoClassDefFoundError, и поймать это внутри самого моста уже поздно. Тот же
 * приём, что у {@code LuckPermsSupport} в core.
 *
 * <p>Имена классов здесь строками, а не через препроцессор: у API два разных
 * пакета в разных поколениях, но {@code Class.forName} принимает и то и другое,
 * ничего при этом не загружая.
 */
final class ModPlaceholders {

    private ModPlaceholders() {}

    static void register(ProfileCache profiles) {
        if (!available()) {
            AgentRuntime.LOG.info("Text Placeholder API not found — %noro:…% placeholders are off");
            return;
        }
        try {
            PlaceholderBridge.register(profiles);
            AgentRuntime.LOG.info("Registered %noro:…% placeholders");
        } catch (Throwable e) {
            // Плейсхолдеры — украшение таба и чата, а не условие работы сервера.
            AgentRuntime.LOG.warn("Text Placeholder API present but unusable: {}", e.toString());
        }
    }

    private static boolean available() {
        return present("eu.pb4.placeholders.api.Placeholders") || present("eu.pb4.placeholders.PlaceholderAPI");
    }

    /** `initialize = false`: класс не инициализируем, только проверяем наличие. */
    private static boolean present(String name) {
        try {
            Class.forName(name, false, ModPlaceholders.class.getClassLoader());
            return true;
        } catch (Throwable e) {
            return false;
        }
    }
}
