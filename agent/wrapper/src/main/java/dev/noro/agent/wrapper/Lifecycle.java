package dev.noro.agent.wrapper;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import org.slf4j.Logger;

/**
 * Однопоточный исполнитель переходов состояния сервера.
 *
 * <p>Синхронизация на общей блокировке здесь не работает: остановка ждёт, пока
 * процесс сохранится и выйдет, а наблюдатель за выходом в этот же момент хочет
 * объявить результат — на одном мониторе они встали бы друг против друга.
 * Очередь из одного потока даёт тот же порядок без взаимной блокировки.
 */
final class Lifecycle implements AutoCloseable {

    private final ExecutorService executor =
            Executors.newSingleThreadExecutor(r -> new Thread(r, "noro-supervisor"));

    /** Выполнить и дождаться, пробросив исходную ошибку вызывающему. */
    void run(ThrowingRunnable action) throws Exception {
        try {
            executor.submit(() -> {
                        action.run();
                        return null;
                    })
                    .get();
        } catch (ExecutionException e) {
            Throwable cause = e.getCause();
            throw cause instanceof Exception ex ? ex : new IllegalStateException(cause);
        }
    }

    /** Выполнить в фоне: некому докладывать об ошибке, кроме лога. */
    void submit(ThrowingRunnable action, Logger log, String what) {
        executor.submit(() -> {
            try {
                action.run();
            } catch (Exception e) {
                log.error("{} failed: {}", what, e.getMessage());
            }
        });
    }

    @Override
    public void close() {
        executor.shutdownNow();
    }

    @FunctionalInterface
    interface ThrowingRunnable {
        void run() throws Exception;
    }
}
