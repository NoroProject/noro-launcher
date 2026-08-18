package dev.noro.agent.core;

import com.google.gson.Gson;
import java.net.http.WebSocket;
import java.util.concurrent.CompletionStage;
import java.util.function.Consumer;
import org.slf4j.Logger;

/**
 * Приём кадров живого канала.
 *
 * <p>Отдельно от {@link AgentLink}, чтобы та осталась про соединение: разбор
 * приходящего растёт вместе с протоколом, а переподключение — нет.
 *
 * <p>Текст приходит по частям — {@code WebSocket} этого не скрывает, и склейка
 * здесь не оптимизация, а условие того, чтобы JSON вообще разобрался.
 */
final class LinkReader implements WebSocket.Listener {

    private final StringBuilder buffer = new StringBuilder();
    private final Gson gson;
    private final AgentLink.Listener listener;
    /** Соединение кончилось: причина — для лога, решение о повторе не здесь. */
    private final Consumer<String> onDown;

    private final Logger log;

    LinkReader(Gson gson, AgentLink.Listener listener, Consumer<String> onDown, Logger log) {
        this.gson = gson;
        this.listener = listener;
        this.onDown = onDown;
        this.log = log;
    }

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
        onDown.accept("closed: " + reason);
        return null;
    }

    @Override
    public void onError(WebSocket ws, Throwable error) {
        onDown.accept(error.getMessage());
    }

    private void dispatch(String frame) {
        try {
            gson.fromJson(frame, LinkFrame.class).deliver(listener);
        } catch (RuntimeException e) {
            // Непонятный кадр — не повод рвать канал: мастер мог уехать вперёд
            // по версии, а остальные кадры мы понимаем.
            log.warn("Skipping a frame from master: {}", e.getMessage());
        }
    }
}
