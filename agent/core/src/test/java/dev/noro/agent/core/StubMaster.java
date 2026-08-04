package dev.noro.agent.core;

import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.time.Duration;

/**
 * Мастер-заглушка на JDK-сервере: даёт прогнать {@link MasterClient} целиком —
 * с заголовком авторизации, кодами ответа и разбором тела, — не поднимая
 * настоящий мастер и не заводя ни одной тестовой зависимости.
 */
final class StubMaster implements AutoCloseable {

    /** Тело из спеки, слово в слово: на нём и проверяется разбор snake_case. */
    static final String PLAYER_JSON =
            """
            {
              "uuid": "95bf010b-8e9f-55d5-8f1c-766c624ab7e0",
              "username": "dalynkaa",
              "banned": false,
              "allowed": true,
              "roles": [
                { "name": "admin", "display_name": "Админ", "lp_group": "admin",
                  "color": "#ff8c82", "icon": "★", "sort_order": 100 }
              ],
              "skin_url": "http://example/api/textures/default-skin",
              "cape_url": null,
              "lp_groups": ["admin", "player"],
              "permissions": ["noro.admin.*", "servercore.command.settings",
                              "prefix.100.§x§f§f§8§c§8§2★§r"]
            }
            """;

    static final String SECRET = "noroagent_" + "ab".repeat(32);

    private final HttpServer server;

    String lastAuthHeader;
    String lastBody;
    String lastPath;

    private StubMaster(HttpServer server) {
        this.server = server;
    }

    static StubMaster start(int status, String responseBody) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
        StubMaster stub = new StubMaster(server);
        server.createContext("/", exchange -> stub.handle(exchange, status, responseBody));
        server.start();
        return stub;
    }

    private void handle(HttpExchange exchange, int status, String responseBody) throws IOException {
        lastAuthHeader = exchange.getRequestHeaders().getFirst("Authorization");
        lastPath = exchange.getRequestURI().getPath();
        lastBody = new String(exchange.getRequestBody().readAllBytes(), StandardCharsets.UTF_8);

        byte[] payload = responseBody.getBytes(StandardCharsets.UTF_8);
        exchange.getResponseHeaders().add("Content-Type", "application/json");
        exchange.sendResponseHeaders(status, payload.length);
        try (var out = exchange.getResponseBody()) {
            out.write(payload);
        }
    }

    AgentConfig config() {
        return new AgentConfig(
                "http://127.0.0.1:" + server.getAddress().getPort(), SECRET, Duration.ofSeconds(30), true);
    }

    @Override
    public void close() {
        server.stop(0);
    }
}
