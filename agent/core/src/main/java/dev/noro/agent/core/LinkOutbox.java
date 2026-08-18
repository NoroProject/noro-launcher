package dev.noro.agent.core;

import java.net.http.WebSocket;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.function.Supplier;
import org.slf4j.Logger;

/**
 * Очередь исходящих кадров живого канала.
 *
 * <p>Отправка живёт в одном потоке не ради порядка, а по требованию API:
 * {@code sendText} нельзя звать, пока не завершился предыдущий вызов. Класть в
 * очередь можно откуда угодно — из игрового потока в том числе, и это главное:
 * событие входа приходит именно оттуда, и ждать в нём сеть нельзя.
 *
 * <p>Очередь ограничена. Отвалившийся мастер иначе съел бы память кадрами,
 * которые всё равно никому не нужны: мастер сверит состав по heartbeat, а
 * событие получаса давности не расскажет ему ничего нового.
 */
final class LinkOutbox implements AutoCloseable {

    private static final int LIMIT = 500;

    private final LinkedBlockingQueue<String> queue = new LinkedBlockingQueue<>();
    private final Supplier<WebSocket> socket;
    private final Logger log;
    private volatile boolean closed;
    private volatile boolean warnedFull;

    LinkOutbox(Supplier<WebSocket> socket, Logger log) {
        this.socket = socket;
        this.log = log;
    }

    void start() {
        Thread sender = new Thread(this::pump, "noro-agent-link-sender");
        sender.setDaemon(true);
        sender.start();
    }

    void offer(String frame) {
        if (queue.size() >= LIMIT) {
            if (!warnedFull) {
                log.warn("Live link is backed up, dropping outgoing events until it recovers");
                warnedFull = true;
            }
            return;
        }
        warnedFull = false;
        queue.offer(frame);
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
                // Связи нет — кадр теряем осознанно. Догонять мастера старыми
                // событиями незачем: состав онлайна он сверит по heartbeat.
                continue;
            }
            try {
                ws.sendText(frame, true).join();
            } catch (Exception e) {
                log.debug("Outgoing frame dropped: {}", e.getMessage());
            }
        }
    }

    @Override
    public void close() {
        closed = true;
    }
}
