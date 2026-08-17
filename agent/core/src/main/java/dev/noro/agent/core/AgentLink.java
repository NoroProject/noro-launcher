package dev.noro.agent.core;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.WebSocket;
import java.time.Duration;
import java.util.UUID;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import org.slf4j.Logger;

/**
 * Живой канал с мастером: {@code GET /api/agent/link}.
 *
 * <p>Без него бан из панели догонял бы игрока только к следующему входу — то
 * есть уже после того, как тот дописал в чат. Соединение всегда открывает
 * агент: до игровой машины снаружи дороги может и не быть.
 *
 * <p>Канал не обязателен для работы сервера. Оборвался — наказания всё равно
 * применятся: на входе их приносит профиль, а команда получает ответ по HTTP.
 * Поэтому здесь только переподключение с ростом паузы и ни одной попытки
 * что-то остановить.
 */
public final class AgentLink implements AutoCloseable {

    private static final int MAX_BACKOFF_SECONDS = 60;

    /** Что делать с тем, что пришло. Реализует {@link Moderation}. */
    public interface Listener {
        void onPunished(UUID target, String targetName, PunishmentInfo punishment);

        void onRevoked(UUID target, String targetName, String kind, String actorLabel);

        void onMessagesChanged();
    }

    private final AgentConfig config;
    private final MasterHttp http;
    private final Listener listener;
    private final Logger log;

    private final HttpClient client =
            HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(10)).build();
    private final ScheduledExecutorService timer = Executors.newSingleThreadScheduledExecutor(r -> {
        Thread thread = new Thread(r, "noro-agent-link");
        thread.setDaemon(true);
        return thread;
    });

    private volatile WebSocket socket;
    private volatile boolean closed;
    private volatile int backoffSeconds = 2;

    public AgentLink(MasterHttp http, Listener listener, Logger log) {
        this.config = http.config();
        this.http = http;
        this.listener = listener;
        this.log = log;
    }

    public void start() {
        connect();
    }

    private void connect() {
        if (closed) {
            return;
        }
        URI uri = URI.create(config.masterUrl().replaceFirst("^http", "ws") + "/api/agent/link");
        client.newWebSocketBuilder()
                .header("Authorization", "Bearer " + config.secret())
                .buildAsync(uri, new Frames())
                .whenComplete((ws, error) -> {
                    if (error != null) {
                        scheduleReconnect(error.getMessage());
                        return;
                    }
                    socket = ws;
                    backoffSeconds = 2;
                    log.info("Live link to master is up: punishments apply immediately");
                });
    }

    private void scheduleReconnect(String reason) {
        if (closed) {
            return;
        }
        int delay = backoffSeconds;
        backoffSeconds = Math.min(backoffSeconds * 2, MAX_BACKOFF_SECONDS);
        log.warn("Live link is down ({}), retrying in {}s", reason, delay);
        timer.schedule(this::connect, delay, TimeUnit.SECONDS);
    }

    @Override
    public void close() {
        closed = true;
        timer.shutdownNow();
        WebSocket current = socket;
        if (current != null) {
            current.abort();
        }
    }

    /** Разбор кадров. Текст может прийти по частям — WebSocket этого не скрывает. */
    private final class Frames implements WebSocket.Listener {

        private final StringBuilder buffer = new StringBuilder();

        @Override
        public CompletionStage<?> onText(WebSocket ws, CharSequence data, boolean last) {
            buffer.append(data);
            if (last) {
                String frame = buffer.toString();
                buffer.setLength(0);
                dispatch(frame);
            }
            ws.request(1);
            return null;
        }

        @Override
        public CompletionStage<?> onClose(WebSocket ws, int status, String reason) {
            scheduleReconnect("closed: " + reason);
            return null;
        }

        @Override
        public void onError(WebSocket ws, Throwable error) {
            scheduleReconnect(error.getMessage());
        }

        private void dispatch(String frame) {
            try {
                LinkFrame parsed = http.gson().fromJson(frame, LinkFrame.class);
                parsed.deliver(listener);
            } catch (RuntimeException e) {
                // Непонятный кадр — не повод рвать канал: мастер мог уехать
                // вперёд по версии, а остальные кадры мы понимаем.
                log.warn("Skipping a frame from master: {}", e.getMessage());
            }
        }
    }
}
