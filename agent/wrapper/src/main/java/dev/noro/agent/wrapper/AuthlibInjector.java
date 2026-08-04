package dev.noro.agent.wrapper;

import com.google.gson.JsonObject;
import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import org.slf4j.Logger;

/**
 * Доставка authlib-injector и сборка аргумента {@code -javaagent}.
 *
 * <p>Источник тот же, что использует лаунчер, чтобы клиент и сервер жили на
 * одной версии инжектора.
 */
public final class AuthlibInjector {

    private static final String LATEST = "https://authlib-injector.yushi.moe/artifact/latest.json";
    private static final String FILE_NAME = "authlib-injector.jar";

    private final WrapperConfig config;
    private final HttpClient http;
    private final Logger log;

    public AuthlibInjector(WrapperConfig config, Logger log) {
        this.config = config;
        this.log = log;
        this.http = HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(10)).build();
    }

    /**
     * @return готовый аргумент JVM
     */
    public String jvmArg() throws IOException, InterruptedException {
        Path jar = ensureJar();
        // Корень Yggdrasil, а не корень мастера. На этом спотыкаются регулярно:
        // с корнем мастера инжектор не находит ALI-метаданные и молча остаётся
        // без authserver и sessionserver.
        return "-javaagent:" + jar.toAbsolutePath() + "=" + config.yggdrasilRoot();
    }

    private Path ensureJar() throws IOException, InterruptedException {
        Path jar = config.workDir().resolve(FILE_NAME);
        if (Files.isRegularFile(jar)) {
            return jar;
        }

        log.info("Downloading authlib-injector");
        HttpResponse<String> meta = http.send(
                HttpRequest.newBuilder(URI.create(LATEST)).timeout(Duration.ofSeconds(30)).GET().build(),
                HttpResponse.BodyHandlers.ofString());
        if (meta.statusCode() != 200) {
            throw new IOException("authlib-injector metadata: HTTP " + meta.statusCode());
        }
        JsonObject parsed = ArtifactDescriptor.GSON.fromJson(meta.body(), JsonObject.class);
        if (parsed == null || !parsed.has("download_url")) {
            throw new IOException("authlib-injector metadata has no download_url");
        }

        HttpResponse<byte[]> body = http.send(
                HttpRequest.newBuilder(URI.create(parsed.get("download_url").getAsString()))
                        .timeout(Duration.ofMinutes(2))
                        .GET()
                        .build(),
                HttpResponse.BodyHandlers.ofByteArray());
        if (body.statusCode() != 200) {
            throw new IOException("cannot download authlib-injector: HTTP " + body.statusCode());
        }

        Files.createDirectories(jar.getParent());
        Files.write(jar, body.body());
        return jar;
    }
}
