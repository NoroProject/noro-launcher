package dev.noro.agent.wrapper;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.WebSocket;
import java.time.Duration;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import org.slf4j.Logger;

/**
 * Канал управления с мастером. Соединение всегда открывает враппер: до игровой
 * машины снаружи может не быть дороги, и полагаться на неё нельзя. Авторизация —
 * секрет игрового сервера, по нему мастер и понимает, кто пришёл.
 */
public final class ControlLink implements AutoCloseable {

    /** Статус уходит и по расписанию: пока сервер грузится, мастер должен видеть его живым. */
    private static final long STATUS_INTERVAL_SECONDS = 30;
    private static final int MAX_BACKOFF_SECONDS = 60;

    private final WrapperConfig config;
    private final PlatformDetect.Detected detected;
    private final Supervisor supervisor;
    private final RequestPump pump;
    private final Logger log;

    private final HttpClient http =
            HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(10)).build();
    private final Outbox outbox;
    private final ScheduledExecutorService timer =
            Executors.newSingleThreadScheduledExecutor(Threads.daemonFactory("noro-link-timer"));

    private volatile WebSocket socket;
    private volatile boolean closed;
    private volatile int backoffSeconds = 2;

    public ControlLink(
            WrapperConfig config,
            PlatformDetect.Detected detected,
            Supervisor supervisor,
            ControlOps ops,
            Logger log) {
        this.config = config;
        this.detected = detected;
        this.supervisor = supervisor;
        this.log = log;
        // В инициализаторе поля ссылка на socket была бы forward reference.
        this.outbox = new Outbox(() -> socket, log);
        this.pump = new RequestPump(ops, outbox::offer, log);
    }

    public void start() {
        outbox.start();
        timer.scheduleAtFixedRate(
                this::sendStatus, STATUS_INTERVAL_SECONDS, STATUS_INTERVAL_SECONDS, TimeUnit.SECONDS);
        supervisor.onConsole(line -> outbox.offer(ControlFrames.console(line)));
        supervisor.onStatusChange(this::sendStatus);
        connect();
    }

    private void connect() {
        if (closed) {
            return;
        }
        http.newWebSocketBuilder()
                .header("Authorization", "Bearer " + config.secret())
                .connectTimeout(Duration.ofSeconds(15))
                .buildAsync(URI.create(config.masterUrl().replaceFirst("^http", "ws") + "/api/agent/ws"), new Handler())
                .exceptionally(error -> {
                    log.warn("Control link failed to connect: {}", error.getMessage());
                    scheduleReconnect();
                    return null;
                });
    }

    private void scheduleReconnect() {
        if (closed) {
            return;
        }
        int delay = backoffSeconds;
        backoffSeconds = Math.min(backoffSeconds * 2, MAX_BACKOFF_SECONDS);
        log.info("Reconnecting to master in {}s", delay);
        timer.schedule(this::connect, delay, TimeUnit.SECONDS);
    }

    private void sendStatus() {
        outbox.offer(ControlFrames.statusFrame(supervisor.status()));
    }

    @Override
    public void close() {
        closed = true;
        WebSocket ws = socket;
        if (ws != null) {
            ws.sendClose(WebSocket.NORMAL_CLOSURE, "shutting down");
        }
        timer.shutdownNow();
        outbox.close();
        pump.close();
    }

    private final class Handler implements WebSocket.Listener {
        private final StringBuilder buffer = new StringBuilder();

        @Override
        public void onOpen(WebSocket ws) {
            log.info("Control link established");
            backoffSeconds = 2;
            socket = ws;
            // Hello уходит мимо очереди: он обязан быть первым кадром сессии.
            Outbox.sendNow(ws, ControlFrames.hello(config, detected, supervisor.status()));
            ws.request(1);
        }

        @Override
        public CompletionStage<?> onText(WebSocket ws, CharSequence data, boolean last) {
            buffer.append(data);
            if (last) {
                String text = buffer.toString();
                buffer.setLength(0);
                pump.accept(text);
            }
            ws.request(1);
            return null;
        }

        @Override
        public CompletionStage<?> onClose(WebSocket ws, int statusCode, String reason) {
            dropped("closed (" + statusCode + "): " + reason);
            return null;
        }

        @Override
        public void onError(WebSocket ws, Throwable error) {
            dropped("error: " + error.getMessage());
        }

        private void dropped(String why) {
            log.warn("Control link {}", why);
            socket = null;
            scheduleReconnect();
        }
    }
}
