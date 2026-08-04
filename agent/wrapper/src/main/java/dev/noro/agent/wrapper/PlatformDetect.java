package dev.noro.agent.wrapper;

import com.google.gson.JsonObject;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;
import java.util.stream.Stream;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;

/**
 * Определение платформы и версии Minecraft по содержимому серверного каталога.
 *
 * <p>Ни одна из трёх платформ не сообщает этого прямо, поэтому смотрим на следы,
 * которые каждая оставляет: у Fabric это {@code install.properties} в лаунчере,
 * у Paper — {@code version.json} внутри paperclip, а NeoForge вообще запускается
 * не из jar'а, и узнаётся по дереву {@code libraries}.
 */
public final class PlatformDetect {

    public record Detected(Platform platform, String mcVersion) {}

    private PlatformDetect() {}

    public static Detected detect(WrapperConfig config) throws IOException {
        if (config.platformOverride() != null && config.mcVersionOverride() != null) {
            return new Detected(
                    Platform.of(config.platformOverride()), config.mcVersionOverride().strip());
        }

        Detected found = fromNeoForgeLibraries(config.serverDir());
        if (found == null) {
            found = fromJar(config.serverJarPath());
        }
        if (found == null) {
            throw new IOException("cannot detect platform, set platform= and mc-version= in noro-wrapper.properties");
        }
        // Частичный ручной override уточняет автоопределение, а не отменяет его.
        Platform platform = config.platformOverride() != null
                ? Platform.of(config.platformOverride())
                : found.platform();
        String mcVersion =
                config.mcVersionOverride() != null ? config.mcVersionOverride().strip() : found.mcVersion();
        return new Detected(platform, mcVersion);
    }

    /**
     * NeoForge-сервер стартует через {@code @libraries/.../unix_args.txt}, а не из
     * jar'а, поэтому его выдаёт каталог. Версия лоадера {@code 21.1.x} отвечает
     * Minecraft {@code 1.21.1} — так устроена их схема нумерации.
     */
    private static Detected fromNeoForgeLibraries(Path serverDir) throws IOException {
        Path neoforge = serverDir.resolve("libraries/net/neoforged/neoforge");
        if (!Files.isDirectory(neoforge)) {
            return null;
        }
        try (Stream<Path> versions = Files.list(neoforge)) {
            return versions
                    .filter(Files::isDirectory)
                    .map(path -> path.getFileName().toString())
                    .flatMap(version -> {
                        String[] parts = version.split("\\.");
                        return parts.length >= 2
                                ? Stream.of(new Detected(Platform.NEOFORGE, "1." + parts[0] + "." + parts[1]))
                                : Stream.empty();
                    })
                    .findFirst()
                    .orElse(null);
        }
    }

    private static Detected fromJar(Path jar) throws IOException {
        if (!Files.isRegularFile(jar)) {
            return null;
        }
        try (ZipFile zip = new ZipFile(jar.toFile())) {
            // Fabric кладёт в лаунчер install.properties с точной версией игры.
            ZipEntry install = zip.getEntry("install.properties");
            if (install != null) {
                Properties props = new Properties();
                try (InputStream in = zip.getInputStream(install)) {
                    props.load(in);
                }
                String game = props.getProperty("game-version");
                if (game != null && !game.isBlank()) {
                    return new Detected(Platform.FABRIC, game.strip());
                }
            }

            ZipEntry version = zip.getEntry("version.json");
            if (version != null) {
                try (InputStream in = zip.getInputStream(version)) {
                    String json = new String(in.readAllBytes(), StandardCharsets.UTF_8);
                    JsonObject parsed = ArtifactDescriptor.GSON.fromJson(json, JsonObject.class);
                    if (parsed != null && parsed.has("id")) {
                        return new Detected(Platform.PAPER, parsed.get("id").getAsString());
                    }
                }
            }
        }
        return null;
    }
}
