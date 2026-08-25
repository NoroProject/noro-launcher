package dev.noro.client.link;

import com.google.gson.JsonObject;
import dev.noro.client.NoroCore;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.WebSocket;
import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.atomic.AtomicReference;

/**
 * Канал с лаунчером: один WebSocket на loopback.
 *
 * <p>WebSocket берётся из JDK — игра идёт на Java 21, и тащить ради одного
 * сокета зависимость незачем. Токена и URL мастера здесь нет: мод шлёт
 * намерение, к мастеру ходит лаунчер.
 *
 * <p>Про содержимое кадров труба не знает ничего — она раздаёт их функциям.
 */
public final class Bridge {

    private static final HttpClient HTTP = HttpClient.newHttpClient();

    private final AtomicReference<WebSocket> socket = new AtomicReference<>();
    private final List<Feature> features;

    public Bridge(List<Feature> features) {
        this.features = features;
    }

    public boolean connected() {
        return socket.get() != null;
    }

    /**
     * Подключиться, если лаунчер оставил рукопожатие. Файла нет — игру
     * запустили не через лаунчер, и это не ошибка, а обычный случай.
     */
    public void connect(Path gameDir) {
        if (connected()) {
            return;
        }
        Handshake handshake = Handshake.read(gameDir);
        if (handshake == null) {
            return;
        }
        HTTP.newWebSocketBuilder()
                .buildAsync(URI.create("ws://127.0.0.1:" + handshake.port()), new Listener())
                .thenAccept(ws -> {
                    socket.set(ws);
                    send(new Hello(handshake.key(), Handshake.PROTOCOL));
                    features.forEach(f -> f.connected(this));
                })
                .exceptionally(e -> {
                    NoroCore.LOG.warn("канал с лаунчером не открылся: {}", e.toString());
                    return null;
                });
    }

    /** Первый кадр соединения. Общий для всех функций — он про канал, не про них. */
    public record Hello(String key, int protocol) {}

    public void send(Object frame) {
        WebSocket ws = socket.get();
        if (ws != null) {
            ws.sendText(Frames.write(frame), true);
        }
    }

    public void close() {
        WebSocket ws = socket.getAndSet(null);
        if (ws != null) {
            ws.abort();
        }
        features.forEach(Feature::disconnected);
    }

    private void dispatch(String text) {
        JsonObject envelope = Frames.envelope(text);
        String type = Frames.type(envelope);
        if (type == null) {
            NoroCore.LOG.debug("не конверт: {}", text);
            return;
        }
        // Кадр предлагается всем функциям, а не первой согласившейся: свод
        // правил нужен и панели разбора, и игрокским экранам, а «кто первый
        // зарегистрировался, тот и получил» зависело бы от порядка загрузки
        // модов — то есть от случайности.
        boolean taken = false;
        for (Feature feature : features) {
            taken |= feature.accept(type, envelope);
        }
        if (!taken) {
            // Незнакомый кадр не рвёт соединение: мод уезжает со сборкой и
            // живёт у людей дольше, чем эта версия лаунчера.
            NoroCore.LOG.debug("кадр {} никому не подошёл", type);
        }
    }

    /** Кадры собираются целиком: WebSocket отдаёт их кусками. */
    private final class Listener implements WebSocket.Listener {
        private final StringBuilder chunks = new StringBuilder();

        @Override
        public CompletionStage<?> onText(WebSocket ws, CharSequence data, boolean last) {
            chunks.append(data);
            if (last) {
                String text = chunks.toString();
                chunks.setLength(0);
                dispatch(text);
            }
            ws.request(1);
            return null;
        }

        @Override
        public CompletionStage<?> onClose(WebSocket ws, int code, String reason) {
            socket.set(null);
            features.forEach(Feature::disconnected);
            return null;
        }

        @Override
        public void onError(WebSocket ws, Throwable error) {
            NoroCore.LOG.debug("канал оборвался: {}", error.toString());
            socket.set(null);
            features.forEach(Feature::disconnected);
        }
    }
}
