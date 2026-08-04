package dev.noro.agent.core;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.Properties;

/**
 * Настройки агента: куда ходить, чем авторизоваться, как часто отчитываться.
 *
 * <p>Окружение важнее файла: секрет держит ServerWrapper и передаёт его дочернему
 * процессу переменной, чтобы plaintext не размножался по конфигам на диске.
 * Файл — запасной путь для тех, кто ставит агент руками, без враппера.
 */
public record AgentConfig(String masterUrl, String secret, Duration heartbeatInterval, boolean denyOnMasterError) {

    public static final String ENV_URL = "NORO_MASTER_URL";
    public static final String ENV_SECRET = "NORO_AGENT_SECRET";
    public static final String SECRET_PREFIX = "noroagent_";

    /** Мастер считает сервер живым 90 секунд — 30 даёт запас на две потери подряд. */
    private static final Duration DEFAULT_INTERVAL = Duration.ofSeconds(30);

    public AgentConfig {
        if (masterUrl == null || masterUrl.isBlank()) {
            throw new IllegalArgumentException(ENV_URL + " is not set");
        }
        if (secret == null || secret.isBlank()) {
            throw new IllegalArgumentException(ENV_SECRET + " is not set");
        }
        if (!secret.startsWith(SECRET_PREFIX)) {
            throw new IllegalArgumentException("agent secret must start with " + SECRET_PREFIX);
        }
        // Дальше URL везде склеивается с "/api/...", поэтому хвостовой слэш
        // убираем один раз здесь, а не в каждом вызове.
        masterUrl = masterUrl.strip();
        while (masterUrl.endsWith("/")) {
            masterUrl = masterUrl.substring(0, masterUrl.length() - 1);
        }
    }

    /**
     * Читает конфиг: сначала переменные окружения, потом {@code noro-agent.properties}
     * в каталоге конфигурации платформы.
     */
    public static AgentConfig load(Path configFile) {
        Properties props = readIfExists(configFile);
        String url = pick(System.getenv(ENV_URL), props.getProperty("master-url"));
        String secret = pick(System.getenv(ENV_SECRET), props.getProperty("secret"));

        Duration interval = DEFAULT_INTERVAL;
        String rawInterval = props.getProperty("heartbeat-seconds");
        if (rawInterval != null && !rawInterval.isBlank()) {
            interval = Duration.ofSeconds(Long.parseLong(rawInterval.strip()));
        }

        // По умолчанию закрыто: граница доступа — «есть аккаунт на мастере», и
        // молча пускать всех, когда мастер недоступен, значит эту границу снять.
        // Кому дороже аптайм сервера, чем строгость, — ставит false осознанно.
        boolean deny = !"false".equalsIgnoreCase(props.getProperty("deny-on-master-error", "true").strip());

        return new AgentConfig(url, secret, interval, deny);
    }

    private static String pick(String fromEnv, String fromFile) {
        return fromEnv != null && !fromEnv.isBlank() ? fromEnv : fromFile;
    }

    private static Properties readIfExists(Path file) {
        Properties props = new Properties();
        if (file == null || !Files.isRegularFile(file)) {
            return props;
        }
        try (var in = Files.newInputStream(file)) {
            props.load(in);
        } catch (IOException e) {
            throw new UncheckedIOException("cannot read " + file, e);
        }
        return props;
    }

    /** Секрет в логах не печатаем — он равносилен доступу к серверу на мастере. */
    @Override
    public String toString() {
        return "AgentConfig[masterUrl=%s, heartbeat=%ss, denyOnMasterError=%s]"
                .formatted(masterUrl, heartbeatInterval.toSeconds(), denyOnMasterError);
    }
}
