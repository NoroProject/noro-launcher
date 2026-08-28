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
 * One WebSocket to the launcher over loopback.
 *
 * <p>No master token or URL lives here: the mod sends an intent, the launcher
 * talks to the master. The pipe knows nothing about frame contents — it hands
 * them to features.
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
     * Connect if the launcher left a handshake. No file means the game was
     * started without the launcher — ordinary, not an error.
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
                    NoroCore.LOG.warn("launcher channel did not open: {}", e.toString());
                    return null;
                });
    }

    /** First frame on the connection. Shared by all features — it's about the channel, not them. */
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
            NoroCore.LOG.debug("not an envelope: {}", text);
            return;
        }
        // Every feature gets the frame, not just the first taker: the rule book
        // is needed by both the case panel and the player screens, and
        // first-registered-wins would depend on mod load order.
        boolean taken = false;
        for (Feature feature : features) {
            taken |= feature.accept(type, envelope);
        }
        if (!taken) {
            // An unknown frame doesn't drop the connection — the mod ships with
            // the build and outlives this launcher version on people's machines.
            NoroCore.LOG.debug("frame {} matched no feature", type);
        }
    }

    /** Reassembles frames — the WebSocket delivers them in chunks. */
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
            NoroCore.LOG.debug("channel broke: {}", error.toString());
            socket.set(null);
            features.forEach(Feature::disconnected);
        }
    }
}
