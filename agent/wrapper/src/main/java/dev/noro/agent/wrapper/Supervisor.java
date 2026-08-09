package dev.noro.agent.wrapper;

import java.io.IOException;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.function.Consumer;
import org.slf4j.Logger;

/**
 * Жизненный цикл сервера: запуск, остановка, перезапуск, автоподъём.
 *
 * <p>Все переходы состояния идут через {@link Lifecycle} — по одному за раз и в
 * порядке поступления. Наружу торчат только команды и текущий статус.
 */
public final class Supervisor {

    /** Сколько ждём корректного завершения, прежде чем убивать. */
    private static final long STOP_TIMEOUT_SECONDS = 60;

    private final WrapperConfig config;
    private final String javaagentArg;
    private final Logger log;
    private final Lifecycle lifecycle = new Lifecycle();

    private final CopyOnWriteArrayList<Consumer<String>> consoleSinks = new CopyOnWriteArrayList<>();
    private final CopyOnWriteArrayList<Runnable> statusSinks = new CopyOnWriteArrayList<>();

    private volatile ServerProcess current;
    private volatile Integer lastExitCode;
    /** Остановка по команде: автоподъём в этом случае не нужен. */
    private volatile boolean stopping;

    public Supervisor(WrapperConfig config, String javaagentArg, Logger log) {
        this.config = config;
        this.javaagentArg = javaagentArg;
        this.log = log;
    }

    public void onConsole(Consumer<String> sink) {
        consoleSinks.add(sink);
    }

    public void onStatusChange(Runnable sink) {
        statusSinks.add(sink);
    }

    public void start() throws Exception {
        lifecycle.run(this::doStart);
    }

    public void stop() throws Exception {
        lifecycle.run(this::doStop);
    }

    public void restart() throws Exception {
        lifecycle.run(() -> {
            doStop();
            doStart();
        });
    }

    public void kill() throws Exception {
        lifecycle.run(() -> {
            ServerProcess process = current;
            if (process != null) {
                stopping = true;
                process.kill();
            }
        });
    }

    /** Строка в консоль сервера. */
    public void command(String line) {
        ServerProcess process = current;
        if (process == null) {
            throw new IllegalStateException("server is not running");
        }
        process.send(line);
    }

    public Status status() {
        ServerProcess process = current;
        return new Status(
                process != null,
                process != null && process.ready(),
                process == null ? 0 : process.uptimeSeconds(),
                lastExitCode);
    }

    public void shutdown() {
        try {
            stop();
        } catch (Exception e) {
            log.warn("Could not stop the server cleanly: {}", e.getMessage());
        }
        lifecycle.close();
    }

    private void doStart() throws IOException {
        if (current != null) {
            throw new IllegalStateException("server is already running");
        }
        stopping = false;
        ServerProcess process = ServerProcess.start(config, javaagentArg, log, this::publish, this::notifyStatus);
        current = process;
        notifyStatus();
        ExitWatcher.watch(
                process,
                config,
                log,
                () -> stopping,
                code -> {
                    current = null;
                    lastExitCode = code;
                    notifyStatus();
                },
                () -> lifecycle.submit(this::doStart, log, "Automatic restart"));
    }

    private void doStop() throws InterruptedException {
        ServerProcess process = current;
        if (process == null) {
            return;
        }
        stopping = true;
        log.info("Stopping the server");
        process.stopGracefully(STOP_TIMEOUT_SECONDS);
    }

    private void publish(String line) {
        for (Consumer<String> sink : consoleSinks) {
            sink.accept(line);
        }
    }

    private void notifyStatus() {
        for (Runnable sink : statusSinks) {
            sink.run();
        }
    }

    /** Что мастер показывает в админке. */
    public record Status(boolean running, boolean ready, long uptimeSeconds, Integer exitCode) {}
}
