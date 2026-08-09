package dev.noro.agent.wrapper;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.time.Duration;
import org.slf4j.Logger;

/**
 * Установка и обновление jar'а агента.
 *
 * <p>Подпись мастера проверяется до того, как файл окажется в {@code plugins/}
 * или {@code mods/}: агент — исполняемый код, который сервер загрузит без
 * вопросов. Это единственная проверка во враппере, которая действительно
 * что-то гарантирует, — она про канал доставки, а не про клиент игрока.
 */
public final class AgentInstaller {

    private static final String FILE_NAME = "noro-agent.jar";

    private final WrapperConfig config;
    private final HttpClient http;
    private final Ed25519Verify verifier;
    private final Logger log;

    public AgentInstaller(WrapperConfig config, String signingPublicKey, Logger log) {
        this.config = config;
        this.log = log;
        this.http = HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(10)).build();
        this.verifier = new Ed25519Verify(signingPublicKey);
    }

    /** @return путь установленного агента */
    public Path install(PlatformDetect.Detected detected) throws IOException, InterruptedException {
        ArtifactDescriptor descriptor = fetchDescriptor(detected);

        if (!verifier.verify(descriptor.signingBytes(), descriptor.signature())) {
            throw new IOException("agent artifact signature does not match the pinned key — refusing to install");
        }

        Path target = detected.platform().installDir(config.serverDir()).resolve(FILE_NAME);
        if (Files.isRegularFile(target) && descriptor.sha1().equalsIgnoreCase(Sha1.of(Files.readAllBytes(target)))) {
            log.info("Agent is up to date ({})", descriptor.sha1());
            return target;
        }

        byte[] jar = download(descriptor.url());
        String actual = Sha1.of(jar);
        if (!descriptor.sha1().equalsIgnoreCase(actual)) {
            // Подпись покрывает sha1, поэтому расхождение здесь — либо битая
            // закачка, либо подмена содержимого по дороге.
            throw new IOException("downloaded agent sha1 " + actual + " != " + descriptor.sha1());
        }

        Files.createDirectories(target.getParent());
        Path tmp = target.resolveSibling(FILE_NAME + ".tmp");
        Files.write(tmp, jar);
        Files.move(tmp, target, StandardCopyOption.REPLACE_EXISTING);
        log.info("Installed agent {} for {} {}", descriptor.sha1(), detected.platform().id(), detected.mcVersion());
        return target;
    }

    private ArtifactDescriptor fetchDescriptor(PlatformDetect.Detected detected)
            throws IOException, InterruptedException {
        String url = "%s/api/agent/artifact?platform=%s&mc=%s"
                .formatted(config.masterUrl(), detected.platform().id(), detected.mcVersion());
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(url))
                .timeout(Duration.ofSeconds(15))
                .header("Authorization", "Bearer " + config.secret())
                .header("Accept", "application/json")
                .GET()
                .build();

        HttpResponse<String> response = http.send(request, HttpResponse.BodyHandlers.ofString());
        if (response.statusCode() != 200) {
            throw new IOException("master returned HTTP " + response.statusCode() + " for " + url + ": "
                    + response.body());
        }
        return ArtifactDescriptor.GSON.fromJson(response.body(), ArtifactDescriptor.class);
    }

    private byte[] download(String url) throws IOException, InterruptedException {
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(url))
                .timeout(Duration.ofMinutes(2))
                .GET()
                .build();
        HttpResponse<byte[]> response = http.send(request, HttpResponse.BodyHandlers.ofByteArray());
        if (response.statusCode() != 200) {
            throw new IOException("cannot download agent: HTTP " + response.statusCode());
        }
        return response.body();
    }
}
