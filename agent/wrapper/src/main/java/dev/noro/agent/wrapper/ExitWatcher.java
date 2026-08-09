package dev.noro.agent.wrapper;

import org.slf4j.Logger;

/**
 * Ждёт выхода процесса и решает, поднимать ли сервер снова.
 *
 * <p>Отдельно от супервизора, потому что это политика, а не механика: «упал
 * через две секунды» и «упал через сутки» — разные события, и различать их
 * должно одно место.
 */
final class ExitWatcher {

    /** Упал раньше — значит не «упал», а «не стартует»; поднимать бесполезно. */
    private static final long CRASH_LOOP_SECONDS = 60;

    private ExitWatcher() {}

    /**
     * @param expected  остановка была по команде — тогда подниматься не нужно
     * @param onExit    вызывается с кодом возврата до решения о перезапуске
     * @param restart   как поднять сервер снова
     */
    static void watch(
            ServerProcess process,
            WrapperConfig config,
            Logger log,
            java.util.function.BooleanSupplier expected,
            java.util.function.IntConsumer onExit,
            Runnable restart) {
        Thread watcher = new Thread(
                () -> {
                    int code;
                    try {
                        code = process.waitFor();
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        return;
                    }
                    long uptime = process.uptimeSeconds();
                    log.info("Server exited with code {} after {}s", code, uptime);
                    onExit.accept(code);

                    if (expected.getAsBoolean() || !config.restartOnCrash()) {
                        return;
                    }
                    if (uptime >= CRASH_LOOP_SECONDS) {
                        log.warn("Server died on its own — restarting");
                        restart.run();
                    } else {
                        log.error("Server died after {}s — not restarting, this looks like a boot failure", uptime);
                    }
                },
                "noro-server-watch");
        watcher.setDaemon(true);
        watcher.start();
    }
}
