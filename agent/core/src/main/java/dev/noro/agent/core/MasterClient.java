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
import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * HTTP-клиент к API мастера. Знает только про два маршрута и ничего про игру.
 *
 * <p>Авторизация — секретом игрового сервера. {@code server_id} мастер берёт из
 * самого секрета, поэтому агент нигде не сообщает, за какой сервер отчитывается.
 */
public final class MasterClient {

    private static final Duration CONNECT_TIMEOUT = Duration.ofSeconds(5);
    private static final Duration REQUEST_TIMEOUT = Duration.ofSeconds(10);

    private final AgentConfig config;
    private final HttpClient http;
    private final Gson gson;

    public MasterClient(AgentConfig config) {
        this.config = config;
        this.http = HttpClient.newBuilder().connectTimeout(CONNECT_TIMEOUT).build();
        // Мастер отдаёт snake_case; одна политика имён избавляет от @SerializedName
        // на каждом поле обеих моделей.
        this.gson = new GsonBuilder()
                .setFieldNamingPolicy(FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
                .create();
    }

    /**
     * Профиль игрока: роли, группы, бан и доступ — одним запросом.
     *
     * @return пустой {@link Optional}, если игрока нет в базе мастера (404)
     */
    public Optional<PlayerProfile> player(UUID mcUuid) throws IOException, InterruptedException {
        HttpRequest request = authorized("/api/agent/players/" + mcUuid).GET().build();
        HttpResponse<String> response = http.send(request, HttpResponse.BodyHandlers.ofString());

        if (response.statusCode() == 404) {
            return Optional.empty();
        }
        requireOk(response);
        try {
            return Optional.ofNullable(gson.fromJson(response.body(), PlayerProfile.class));
        } catch (JsonSyntaxException e) {
            throw new IOException("master returned malformed player JSON", e);
        }
    }

    /** Сигнал жизни. Мастер считает сервер живым 90 секунд после последнего. */
    public void heartbeat(int online, int maxPlayers, String version) throws IOException, InterruptedException {
        String body = gson.toJson(new Heartbeat(online, maxPlayers, version));
        HttpRequest request = authorized("/api/agent/heartbeat")
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(body))
                .build();
        requireOk(http.send(request, HttpResponse.BodyHandlers.ofString()));
    }

    /**
     * Каталог узлов прав, зарегистрированных на этом сервере.
     *
     * <p>Перечислить их заранее неоткуда: узлы приносят сами моды, и у каждой
     * сборки набор свой. Мастер по этому списку подсказывает в админке, а не
     * гадает по захардкоженному перечню.
     */
    public void reportNodes(Collection<String> nodes) throws IOException, InterruptedException {
        String body = gson.toJson(new Nodes(List.copyOf(nodes)));
        HttpRequest request = authorized("/api/agent/nodes")
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(body))
                .build();
        requireOk(http.send(request, HttpResponse.BodyHandlers.ofString()));
    }

    private HttpRequest.Builder authorized(String path) {
        return HttpRequest.newBuilder()
                .uri(URI.create(config.masterUrl() + path))
                .timeout(REQUEST_TIMEOUT)
                .header("Authorization", "Bearer " + config.secret())
                .header("Accept", "application/json");
    }

    private static void requireOk(HttpResponse<String> response) throws IOException {
        int code = response.statusCode();
        if (code / 100 == 2) {
            return;
        }
        // 401 стоит назвать своим именем: это единственная ошибка, которую
        // администратор чинит сам — перевыпуском секрета в админке.
        if (code == 401 || code == 403) {
            throw new IOException("master rejected the agent secret (HTTP " + code
                    + ") — reissue it in the build admin panel, section Game servers");
        }
        throw new IOException("master returned HTTP " + code + ": " + response.body());
    }

    /** Тело heartbeat. Имена полей сериализуются политикой Gson в snake_case. */
    private record Heartbeat(int online, int maxPlayers, String version) {}

    /** Тело отчёта об узлах прав. */
    private record Nodes(List<String> nodes) {}
}
