package dev.noro.agent.core;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.WebSocket;
import java.time.Duration;
import java.util.UUID;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import org.slf4j.Logger;

/**
 * Живой канал с мастером: {@code GET /api/agent/link}.
 *
 * <p>Вниз идут наказания — без канала бан из панели догонял бы игрока только к
 * следующему входу, то есть уже после того, как тот дописал в чат. Вверх идут
 * события игры, которых мастеру иначе не увидеть: вход, выход, зависание.
 * Соединение всегда открывает агент: до игровой машины снаружи дороги может и
 * не быть.
 *
 * <p>Канал не обязателен для работы сервера. Оборвался — наказания всё равно
 * применятся: на входе их приносит профиль, а команда получает ответ по HTTP.
 * И наоборот: ни одно решение агента не ждёт ответа мастера отсюда. Поэтому
 * здесь только переподключение с ростом паузы и ни одной попытки что-то
 * остановить.
 */
public final class AgentLink implements AutoCloseable {

    private static final int MAX_BACKOFF_SECONDS = 60;

    /** Что делать с тем, что пришло. Реализует {@link Moderation}. */
    public interface Listener {
        void onPunished(UUID target, String targetName, PunishmentInfo punishment);

        void onRevoked(UUID target, String targetName, String kind, String actorLabel);

        void onMessagesChanged();

        default void onFiltersChanged() {}

        default void onRestartNotice(int seconds, String reason) {}

        /** Роли или права изменились. {@code null} — перечитать всех. */
        void onProfileChanged(UUID uuid);

        default void onKick(UUID target, String message) {}

        default void onTell(UUID target, String message) {}

        default void onAnnounce(String message) {}

        default void onMaintenanceStart(int countdownSeconds, String reason) {}

        default void onMaintenanceCancel() {}

        /** Дело отдали модератору — включить режим разбора. */
        default void onCaseAssigned(CaseSession session, UUID moderator) {}

        /** Дело отпустили или закрыли. */
        default void onCaseFinished(UUID moderator, UUID caseId, boolean closed) {}

        default void onCaseChatRequest(UUID caseId, int beforeSecs) {}

        default void onCaseInventoryRequest(UUID caseId, UUID target) {}
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

    private final LinkOutbox outbox;
    private final AgentEvents events;

    private volatile WebSocket socket;
    private volatile boolean closed;
    private volatile int backoffSeconds = 2;

    public AgentLink(MasterHttp http, Listener listener, Logger log) {
        this.config = http.config();
        this.http = http;
        this.listener = listener;
        this.log = log;
        this.outbox = new LinkOutbox(() -> socket, log);
        this.events = new AgentEvents(outbox, http.gson());
    }

    /** Как сообщить мастеру о случившемся в игре. */
    public AgentEvents events() {
        return events;
    }

    public void start() {
        outbox.start();
        connect();
    }

    private void connect() {
        if (closed) {
            return;
        }
        URI uri = URI.create(config.masterUrl().replaceFirst("^http", "ws") + "/api/agent/link");
        client.newWebSocketBuilder()
                .header("Authorization", "Bearer " + config.secret())
                .buildAsync(uri, new LinkReader(http.gson(), listener, this::scheduleReconnect, log))
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
        // Мёртвый сокет держать нельзя: очередь исходящих отдаёт в него кадры и
        // теряла бы их молча, считая связь живой.
        socket = null;
        int delay = backoffSeconds;
        backoffSeconds = Math.min(backoffSeconds * 2, MAX_BACKOFF_SECONDS);
        log.warn("Live link is down ({}), retrying in {}s", reason, delay);
        timer.schedule(this::connect, delay, TimeUnit.SECONDS);
    }

    @Override
    public void close() {
        closed = true;
        outbox.close();
        timer.shutdownNow();
        WebSocket current = socket;
        if (current != null) {
            current.abort();
        }
    }
}
