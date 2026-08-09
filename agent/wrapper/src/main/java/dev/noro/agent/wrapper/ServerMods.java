package dev.noro.agent.wrapper;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
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
 * Установка модов на игровой сервер.
 *
 * <p>Файл качает враппер, а не мастер по сокету: он уже лежит в сторе мастера,
 * адресуется своим sha1 и раздаётся обычным HTTP с докачкой. Гонять тот же jar
 * через WebSocket ради того же результата смысла нет.
 *
 * <p>Подпись здесь не проверяется — в отличие от jar'а агента. Мод выбрал
 * администратор из каталога, мастер его не подписывал и подписать не может;
 * гарантия тут одна и настоящая — sha1 совпал с тем, что мастер положил в стор.
 */
final class ServerMods {

    private final ServerPaths paths;
    private final Logger log;
    private final HttpClient http =
            HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(15)).build();

    ServerMods(ServerPaths paths, Logger log) {
        this.paths = paths;
        this.log = log;
    }

    JsonElement install(String url, String sha1, String filename, String dir)
            throws IOException, InterruptedException {
        Path target = paths.resolve(dir + "/" + filename);

        JsonObject result = new JsonObject();
        result.addProperty("path", paths.relativize(target));
        if (Files.isRegularFile(target) && sha1.equalsIgnoreCase(Sha1.of(Files.readAllBytes(target)))) {
            log.info("Mod {} is already installed", filename);
            result.addProperty("changed", false);
            return result;
        }

        byte[] jar = download(url);
        String actual = Sha1.of(jar);
        if (!sha1.equalsIgnoreCase(actual)) {
            // Битая закачка или подмена по дороге — в mods/ такое класть нельзя.
            throw new IOException("downloaded sha1 " + actual + " != " + sha1);
        }

        Files.createDirectories(target.getParent());
        Path tmp = target.resolveSibling(filename + ".noro-tmp");
        Files.write(tmp, jar);
        Files.move(tmp, target, StandardCopyOption.REPLACE_EXISTING);
        log.info("Installed {} into {}", filename, dir);

        result.addProperty("changed", true);
        result.addProperty("size", jar.length);
        // Лоадер читает каталог модов один раз при старте: без рестарта файл
        // на диске есть, а в игре его нет. Админке это надо показать.
        result.addProperty("restart_required", true);
        return result;
    }

    private byte[] download(String url) throws IOException, InterruptedException {
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(url))
                .timeout(Duration.ofMinutes(5))
                .GET()
                .build();
        HttpResponse<byte[]> response = http.send(request, HttpResponse.BodyHandlers.ofByteArray());
        if (response.statusCode() != 200) {
            throw new IOException("cannot download the mod: HTTP " + response.statusCode());
        }
        return response.body();
    }
}
