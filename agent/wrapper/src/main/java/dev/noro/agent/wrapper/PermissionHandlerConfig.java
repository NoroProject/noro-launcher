package dev.noro.agent.wrapper;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.regex.Pattern;
import org.slf4j.Logger;

/**
 * Переключает Forge/NeoForge на обработчик прав агента.
 *
 * <p>Одной регистрации обработчика мало: лоадер выбирает его по строке в
 * `config/{neo}forge-server.toml`, и по умолчанию там встроенный. Без этой
 * правки права мастера игнорируются **молча** — ни ошибки, ни предупреждения,
 * просто ничего не работает. Такую ловушку оставлять администратору нельзя,
 * поэтому её закрывает установщик.
 */
public final class PermissionHandlerConfig {

    /** Идентификатор обработчика из мода-агента. */
    private static final String OURS = "noro:agent";

    /** Значения, которые ставит сам лоадер и которые не жалко перебить. */
    private static final List<String> DEFAULTS =
            List.of("neoforge:default_handler", "forge:default_handler");

    private static final Pattern LINE =
            Pattern.compile("^(\\s*permissionHandler\\s*=\\s*)\"([^\"]*)\"(.*)$");

    private PermissionHandlerConfig() {}

    /** Правит конфиг под платформу; для Fabric ничего не делает. */
    public static void ensure(WrapperConfig config, Platform platform, Logger log) {
        if (platform == Platform.FABRIC || platform == Platform.PAPER) {
            return;
        }
        Path file = config
                .serverDir()
                .resolve("config")
                .resolve(platform == Platform.NEOFORGE ? "neoforge-server.toml" : "forge-server.toml");
        if (!Files.isRegularFile(file)) {
            // Файл появляется после первого запуска сервера — на следующем
            // старте враппер его и поправит.
            return;
        }
        try {
            apply(file, log);
        } catch (IOException e) {
            log.warn("Cannot update {}: {}", file.getFileName(), e.getMessage());
        }
    }

    private static void apply(Path file, Logger log) throws IOException {
        List<String> lines = Files.readAllLines(file, StandardCharsets.UTF_8);
        for (int i = 0; i < lines.size(); i++) {
            var matcher = LINE.matcher(lines.get(i));
            if (!matcher.matches()) {
                continue;
            }
            String current = matcher.group(2);
            if (OURS.equals(current)) {
                return;
            }
            if (!DEFAULTS.contains(current)) {
                // Кто-то выбрал обработчик осознанно — например, поставил
                // LuckPerms. Перебивать чужой выбор установщик не вправе.
                log.warn(
                        "Permission handler is set to {} — leaving it. Master permissions stay off"
                                + " until it is {}",
                        current,
                        OURS);
                return;
            }
            lines.set(i, matcher.group(1) + '"' + OURS + '"' + matcher.group(3));
            Files.write(file, lines, StandardCharsets.UTF_8);
            log.info("Switched permission handler to {} in {}", OURS, file.getFileName());
            return;
        }
        log.warn("No permissionHandler entry in {} — master permissions will be ignored", file.getFileName());
    }
}
