package dev.noro.agent.wrapper;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import java.net.http.WebSocket;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.function.Supplier;
import org.slf4j.Logger;

/**
 * Очередь исходящих кадров.
 *
 * <p>Отправка живёт в одном потоке не для порядка, а по требованию API:
 * {@code sendText} нельзя вызывать, пока не завершился предыдущий. Очередь
 * ограничена — иначе отвалившийся мастер съел бы память логами сервера.
 */
final class Outbox implements AutoCloseable {

    private static final Gson GSON = new Gson();
    private static final int LIMIT = 2000;

    private final LinkedBlockingQueue<String> queue = new LinkedBlockingQueue<>();
    private final Supplier<WebSocket> socket;
    private final Logger log;
    private volatile boolean closed;

    Outbox(Supplier<WebSocket> socket, Logger log) {
        this.socket = socket;
        this.log = log;
    }

    void start() {
        Threads.daemonFactory("noro-link-sender").newThread(this::pump).start();
    }

    void offer(JsonObject frame) {
        if (queue.size() < LIMIT) {
            queue.offer(GSON.toJson(frame));
        }
    }

    /** Отправить немедленно, мимо очереди: так уходит только hello. */
    static void sendNow(WebSocket ws, JsonObject frame) {
        ws.sendText(GSON.toJson(frame), true).join();
    }

    private void pump() {
        while (!closed) {
            String frame;
            try {
                frame = queue.take();
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                return;
            }
            WebSocket ws = socket.get();
            if (ws == null) {
                // Связи нет — кадр теряем осознанно: догонять мастера старым
                // хвостом консоли незачем, а статус уйдёт заново с hello.
                continue;
            }
            try {
                ws.sendText(frame, true).join();
            } catch (Exception e) {
                log.debug("Frame dropped: {}", e.getMessage());
            }
        }
    }

    @Override
    public void close() {
        closed = true;
    }
}
