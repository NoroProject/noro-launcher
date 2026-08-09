package dev.noro.agent.wrapper;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.function.Consumer;
import org.slf4j.Logger;

/**
 * Разбор запросов мастера и отправка ответов.
 *
 * <p>Операции идут в своих потоках: установка мода или архив мира занимают
 * минуты, а реактор WebSocket всё это время должен читать сокет — иначе
 * следующая команда не дойдёт, и мастер решит, что враппер умер.
 */
final class RequestPump implements AutoCloseable {

    private final ControlOps ops;
    private final Consumer<JsonObject> sink;
    private final Logger log;
    private final ExecutorService workers =
            Executors.newFixedThreadPool(2, Threads.daemonFactory("noro-link-worker"));

    RequestPump(ControlOps ops, Consumer<JsonObject> sink, Logger log) {
        this.ops = ops;
        this.sink = sink;
        this.log = log;
    }

    void accept(String text) {
        JsonObject message;
        try {
            message = JsonParser.parseString(text).getAsJsonObject();
        } catch (RuntimeException e) {
            log.warn("Master sent something unparseable: {}", e.getMessage());
            return;
        }
        if (!"request".equals(string(message, "type"))) {
            return;
        }
        long id = message.get("id").getAsLong();
        String op = string(message, "op");
        JsonObject args = message.has("args") && message.get("args").isJsonObject()
                ? message.getAsJsonObject("args")
                : new JsonObject();
        workers.submit(() -> run(id, op, args));
    }

    private void run(long id, String op, JsonObject args) {
        try {
            JsonElement data = ops.execute(op, args);
            sink.accept(ControlFrames.ok(id, data));
        } catch (Exception e) {
            log.warn("Operation {} failed: {}", op, e.toString());
            sink.accept(ControlFrames.failed(id, e.getMessage() == null ? e.toString() : e.getMessage()));
        }
    }

    private static String string(JsonObject json, String key) {
        return json.has(key) ? json.get(key).getAsString() : "";
    }

    @Override
    public void close() {
        workers.shutdownNow();
    }
}
