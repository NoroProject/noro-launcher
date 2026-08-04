package dev.noro.agent.core;

import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import org.slf4j.Logger;

/**
 * Периодический сигнал жизни мастеру. Первый уходит сразу при старте, дальше —
 * раз в {@link AgentConfig#heartbeatInterval()}.
 */
public final class HeartbeatTask implements AutoCloseable {

    private final MasterClient client;
    private final ServerStatus status;
    private final AgentConfig config;
    private final Logger log;
    private final ScheduledExecutorService scheduler;

    /** Чтобы не заваливать лог одинаковой ошибкой раз в 30 секунд. */
    private boolean lastAttemptFailed;

    public HeartbeatTask(MasterClient client, ServerStatus status, AgentConfig config, Logger log) {
        this.client = client;
        this.status = status;
        this.config = config;
        this.log = log;
        this.scheduler = Executors.newSingleThreadScheduledExecutor(r -> {
            // Демон: иначе поток удержит JVM после остановки сервера.
            Thread thread = new Thread(r, "noro-agent-heartbeat");
            thread.setDaemon(true);
            return thread;
        });
    }

    public void start() {
        long period = config.heartbeatInterval().toSeconds();
        scheduler.scheduleAtFixedRate(this::beat, 0, period, TimeUnit.SECONDS);
    }

    private void beat() {
        // Вылетевшее исключение отменяет расписание насовсем — сервер потом
        // числится мёртвым до перезапуска. Поэтому ловим здесь всё.
        try {
            client.heartbeat(status.online(), status.maxPlayers(), status.version());
            if (lastAttemptFailed) {
                log.info("Heartbeat restored, master is reachable again");
                lastAttemptFailed = false;
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        } catch (Exception e) {
            if (!lastAttemptFailed) {
                log.warn("Heartbeat failed, the server will show as offline: {}", e.getMessage());
                lastAttemptFailed = true;
            }
        }
    }

    @Override
    public void close() {
        scheduler.shutdownNow();
    }
}
