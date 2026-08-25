package dev.noro.agent.wrapper;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import org.slf4j.Logger;

/**
 * Спросить у мастера пак плашек и прописать его в {@code server.properties}.
 *
 * <p>Делается до запуска сервера: конфиг он читает один раз при старте, и
 * дописанная позже строка осталась бы незамеченной до следующего рестарта.
 *
 * <p>Строка ставится, только когда выдавать пак больше некому. Ванильная выдача
 * из `server.properties` уходит **всем** в фазе настройки, и клиент на каждом
 * входе перезагружает ресурсы — даже тому, у кого пак уже стоит из сборки
 * лаунчера. Поэтому обычный путь — поштучная выдача агентом, а она умеет
 * пропускать тех, у кого пак и так есть.
 *
 * <p>Мастер недоступен — не беда: сервер поднимется без строки, а плашки доедут
 * выдачей агента тем, чей клиент это умеет.
 */
final class PrefixPackConfig {

    private static final Gson GSON = new Gson();

    private PrefixPackConfig() {}

    static void ensure(WrapperConfig config, Logger log) {
        try {
            HttpClient http = HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(5)).build();
            HttpRequest request = HttpRequest.newBuilder()
                    .uri(URI.create(config.masterUrl() + "/api/agent/prefix-pack"))
                    .timeout(Duration.ofSeconds(10))
                    .header("Authorization", "Bearer " + config.secret())
                    .header("Accept", "application/json")
                    .GET()
                    .build();
            HttpResponse<String> response = http.send(request, HttpResponse.BodyHandlers.ofString());
            if (response.statusCode() != 200) {
                log.info("Prefix pack is not configured on the master (HTTP {})", response.statusCode());
                return;
            }
            JsonObject body = GSON.fromJson(response.body(), JsonObject.class);
            String url = body.has("url") ? body.get("url").getAsString() : "";
            String sha1 = body.has("sha1") ? body.get("sha1").getAsString() : "";
            // Ничего не прописываем: пак раздаёт агент поштучно. Строки при
            // этом чистим — прежняя выдача всем как раз и заставляла клиент
            // перезагружать ресурсы на каждом входе.
            ServerResourcePack.apply(config.serverDir(), "", "", log);
            if (!url.isEmpty()) {
                log.info("Prefix pack {} is handed out per player, not via server.properties", sha1);
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        } catch (Exception e) {
            log.warn("Cannot ask the master for the prefix pack: {}", e.toString());
        }
    }
}
