package dev.noro.agent.wrapper;

import com.google.gson.JsonObject;
import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import org.slf4j.Logger;

/**
 * Откуда враппер берёт публичный ключ мастера.
 *
 * <p>Явно заданный в конфиге ключ — настоящий якорь доверия и всегда важнее.
 * Если его нет, ключ берётся у мастера **один раз** и запоминается на диске;
 * дальше сверяется только с запомненным. Это снимает ручной перенос ключа на
 * каждый сервер, из-за которого они и разъезжаются, но не превращает проверку
 * подписи в «мастер сам сказал, что всё хорошо»: подменить ключ можно только в
 * момент первой установки, а не в любой момент потом.
 */
public final class SigningKey {

    private static final String FILE_NAME = "signing-key.pub";

    private SigningKey() {}

    public static String resolve(WrapperConfig config, Logger log) throws IOException, InterruptedException {
        String configured = config.signingPublicKey();
        if (configured != null && !configured.isBlank()) {
            return configured.strip();
        }

        Path stored = config.workDir().resolve(FILE_NAME);
        String fetched = fetch(config);

        if (Files.isRegularFile(stored)) {
            String known = Files.readString(stored, StandardCharsets.UTF_8).strip();
            if (!known.equalsIgnoreCase(fetched)) {
                // Либо у мастера сменили ключ, либо это не тот мастер. Разбираться
                // должен человек: молча принять новый ключ — значит отдать подпись.
                throw new IOException("master signing key changed: pinned " + known + ", got " + fetched
                        + " — verify this was intentional, then update " + stored);
            }
            return known;
        }

        Files.createDirectories(stored.getParent());
        Files.writeString(stored, fetched + "\n", StandardCharsets.UTF_8);
        log.warn("Pinned master signing key {} on first run — from now on a change is a hard error", fetched);
        return fetched;
    }

    private static String fetch(WrapperConfig config) throws IOException, InterruptedException {
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(config.masterUrl() + "/api/agent/pubkey"))
                .timeout(Duration.ofSeconds(15))
                .header("Authorization", "Bearer " + config.secret())
                .header("Accept", "application/json")
                .GET()
                .build();

        HttpResponse<String> response = HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(10))
                .build()
                .send(request, HttpResponse.BodyHandlers.ofString());
        if (response.statusCode() != 200) {
            throw new IOException("cannot fetch signing key: HTTP " + response.statusCode());
        }

        JsonObject parsed = ArtifactDescriptor.GSON.fromJson(response.body(), JsonObject.class);
        if (parsed == null || !parsed.has("public_key")) {
            throw new IOException("master returned no public_key");
        }
        return parsed.get("public_key").getAsString().strip();
    }
}
