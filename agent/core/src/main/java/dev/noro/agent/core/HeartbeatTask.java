package dev.noro.agent.core;

import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import org.slf4j.Logger;

/**
 * Периодический сигнал жизни мастеру. Первый уходит сразу при старте, дальше —
 * раз в {@link AgentConfig#heartbeatInterval()}.
 *
 * <p>Он же замечает зависание. Поток здесь свой, поэтому встал игровой поток
 * или нет — видно: {@link ServerStatus#stalledSeconds()} читается отсюда, а
 * растёт он ровно тогда, когда сервер перестал тикать.
 */
public final class HeartbeatTask implements AutoCloseable {

    /** Дольше этого сервер уже не отвечает игрокам — спорить не с чем. */
    private static final int STALL_SECONDS = 60;

    private final MasterClient client;
    private final ServerStatus status;
    private final AgentConfig config;
    private final Logger log;
    private final ScheduledExecutorService scheduler;

    /** Куда сообщить о зависании. {@code null} — канала нет, останется лог. */
    private volatile AgentEvents events;

    /** Чтобы не заваливать лог одинаковой ошибкой раз в 30 секунд. */
    private boolean lastAttemptFailed;

    /** Об одном зависании сообщаем один раз, а не каждые полминуты. */
    private boolean stallReported;

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

    /**
     * Подключить канал. Отдельно от конструктора: на модах сервер стартует
     * раньше, чем канал поднимется, а сигнал жизни ждать этого не должен.
     */
    public void reportStallsTo(AgentEvents events) {
        this.events = events;
    }

    public void start() {
        long period = config.heartbeatInterval().toSeconds();
        scheduler.scheduleAtFixedRate(this::beat, 0, period, TimeUnit.SECONDS);
    }

    /**
     * Сервер не тикает дольше порога — сказать об этом, пока есть кому.
     *
     * <p>Решение о рестарте принимает мастер: он один знает, включена ли эта
     * политика на сервере, и он же дотянется до враппера.
     */
    private void checkStall() {
        int stalled = status.stalledSeconds();
        if (stalled < STALL_SECONDS) {
            stallReported = false;
            return;
        }
        if (stallReported) {
            return;
        }
        stallReported = true;
        log.error("Server tick has not advanced for {}s", stalled);
        AgentEvents sink = events;
        if (sink != null) {
            sink.tickStall(stalled);
        }
    }

    private void beat() {
        // Вылетевшее исключение отменяет расписание насовсем — сервер потом
        // числится мёртвым до перезапуска. Поэтому ловим здесь всё.
        try {
            checkStall();
            client.heartbeat(status);
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
