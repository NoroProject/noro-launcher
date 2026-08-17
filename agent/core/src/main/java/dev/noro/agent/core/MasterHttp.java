package dev.noro.agent.core;

import com.google.gson.FieldNamingPolicy;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonSyntaxException;
import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.time.Instant;
import java.util.Optional;

/**
 * Транспорт до мастера: авторизация, коды ответа и разбор JSON — в одном месте.
 *
 * <p>Раньше это жило прямо в {@link MasterClient}, но с приходом наказаний
 * клиентов стало два, и копировать в оба обработку 401 значило однажды забыть
 * это сделать. Что именно спрашивать у мастера, здесь не знают.
 */
public final class MasterHttp {

    private static final Duration CONNECT_TIMEOUT = Duration.ofSeconds(5);
    private static final Duration REQUEST_TIMEOUT = Duration.ofSeconds(10);

    private final AgentConfig config;
    private final HttpClient http;
    private final Gson gson;

    public MasterHttp(AgentConfig config) {
        this.config = config;
        this.http = HttpClient.newBuilder().connectTimeout(CONNECT_TIMEOUT).build();
        // Мастер отдаёт snake_case; одна политика имён избавляет от @SerializedName
        // на каждом поле всех моделей. Время — RFC 3339 из chrono, для него у
        // Gson своего разбора нет.
        this.gson = new GsonBuilder()
                .setFieldNamingPolicy(FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
                .registerTypeAdapter(
                        Instant.class,
                        (com.google.gson.JsonDeserializer<Instant>)
                                (json, type, ctx) -> Instant.parse(json.getAsString()))
                .create();
    }

    public AgentConfig config() {
        return config;
    }

    public Gson gson() {
        return gson;
    }

    /** @return пустой {@link Optional}, если мастер ответил 404 */
    public <T> Optional<T> get(String path, Class<T> type) throws IOException, InterruptedException {
        HttpResponse<String> response = send(authorized(path).GET().build());
        if (response.statusCode() == 404) {
            return Optional.empty();
        }
        requireOk(response);
        return Optional.ofNullable(parse(response.body(), type));
    }

    public <T> T post(String path, Object body, Class<T> type) throws IOException, InterruptedException {
        HttpResponse<String> response = send(authorized(path)
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(gson.toJson(body)))
                .build());
        requireOk(response);
        return type == null ? null : parse(response.body(), type);
    }

    public void post(String path, Object body) throws IOException, InterruptedException {
        post(path, body, null);
    }

    private HttpResponse<String> send(HttpRequest request) throws IOException, InterruptedException {
        return http.send(request, HttpResponse.BodyHandlers.ofString());
    }

    private HttpRequest.Builder authorized(String path) {
        return HttpRequest.newBuilder()
                .uri(URI.create(config.masterUrl() + path))
                .timeout(REQUEST_TIMEOUT)
                .header("Authorization", "Bearer " + config.secret())
                .header("Accept", "application/json");
    }

    private <T> T parse(String body, Class<T> type) throws IOException {
        try {
            return gson.fromJson(body, type);
        } catch (JsonSyntaxException e) {
            throw new IOException("master returned malformed JSON", e);
        }
    }

    /**
     * Отказ мастера — это ответ модератору, а не сбой связи: «правило не
     * позволяет столько» приходит именно так, и текст надо донести дословно.
     */
    private static void requireOk(HttpResponse<String> response) throws IOException {
        int code = response.statusCode();
        if (code / 100 == 2) {
            return;
        }
        if (code == 401) {
            throw new IOException("master rejected the agent secret (HTTP 401)"
                    + " — reissue it in the build admin panel, section Game servers");
        }
        throw new MasterRefusedException(code, explain(response.body()));
    }

    /** Мастер отвечает `{"error": "..."}`; на всё остальное отдаём тело как есть. */
    private static String explain(String body) {
        try {
            var object = com.google.gson.JsonParser.parseString(body).getAsJsonObject();
            if (object.has("error")) {
                return strip(object.get("error").getAsString());
            }
        } catch (RuntimeException ignored) {
            // Не JSON — значит показываем сырое тело, оно всё равно информативнее.
        }
        return body;
    }

    /**
     * Мастер печатает вид ошибки в самом тексте («forbidden: …»). В чате это
     * шум: код ответа модератору ничего не говорит, а причина — говорит.
     */
    private static String strip(String message) {
        for (String prefix : new String[] {"forbidden: ", "bad request: ", "not found: ", "conflict: "}) {
            if (message.startsWith(prefix)) {
                return message.substring(prefix.length());
            }
        }
        return message;
    }
}
