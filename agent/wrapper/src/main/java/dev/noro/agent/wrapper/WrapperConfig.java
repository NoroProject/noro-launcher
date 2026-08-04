package dev.noro.agent.wrapper;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import java.util.Properties;

/**
 * Конфигурация враппера — единственное место, где лежит секрет агента.
 *
 * <p>Агент внутри сервера получает его переменной окружения от враппера, поэтому
 * plaintext не расползается по конфигам в {@code plugins/} и {@code mods/}.
 */
public record WrapperConfig(
        String masterUrl,
        String secret,
        String signingPublicKey,
        Path serverDir,
        String serverJar,
        String javaBin,
        List<String> jvmArgs,
        List<String> serverArgs,
        String platformOverride,
        String mcVersionOverride) {

    public static WrapperConfig load(Path file) {
        Properties props = new Properties();
        try (var in = Files.newInputStream(file)) {
            props.load(in);
        } catch (IOException e) {
            throw new UncheckedIOException("cannot read " + file, e);
        }

        Path serverDir = Path.of(props.getProperty("server-dir", ".")).toAbsolutePath().normalize();
        return new WrapperConfig(
                required(props, "master-url"),
                required(props, "secret"),
                // Необязателен: пустой означает «взять у мастера и запомнить»
                // (см. SigningKey). Значения по умолчанию нет и быть не должно —
                // дефолт здесь молча доверял бы dev-ключу.
                props.getProperty("signing-public-key", "").strip(),
                serverDir,
                required(props, "server-jar"),
                props.getProperty("java", "java"),
                words(props.getProperty("jvm-args", "-Xmx4G")),
                words(props.getProperty("server-args", "nogui")),
                props.getProperty("platform"),
                props.getProperty("mc-version"));
    }

    /** Путь к jar сервера — он же вход для определения платформы. */
    public Path serverJarPath() {
        return serverDir.resolve(serverJar);
    }

    /** Рабочий каталог враппера внутри сервера: authlib-injector и служебное. */
    public Path workDir() {
        return serverDir.resolve("noro");
    }

    public String yggdrasilRoot() {
        // authlib-injector ждёт корень Yggdrasil-API, а не корень мастера: от
        // этого адреса он строит пути authserver и sessionserver.
        return masterUrl + "/api/yggdrasil";
    }

    private static String required(Properties props, String key) {
        String value = props.getProperty(key);
        if (value == null || value.isBlank()) {
            throw new IllegalArgumentException("noro-wrapper.properties: " + key + " is required");
        }
        return value.strip();
    }

    private static List<String> words(String raw) {
        if (raw == null || raw.isBlank()) {
            return List.of();
        }
        return Arrays.stream(raw.strip().split("\\s+")).toList();
    }

    public WrapperConfig {
        masterUrl = masterUrl.strip();
        while (masterUrl.endsWith("/")) {
            masterUrl = masterUrl.substring(0, masterUrl.length() - 1);
        }
    }
}
