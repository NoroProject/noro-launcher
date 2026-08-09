package dev.noro.agent.wrapper;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * Кадры протокола управления. Только сборка JSON — ни сокета, ни состояния.
 *
 * <p>Отдельно от канала, чтобы форма кадра лежала в одном месте: мастер разбирает
 * её по полям, и расхождение здесь стоит молчаливо потерянной команды.
 */
final class ControlFrames {

    private ControlFrames() {}

    static JsonObject hello(WrapperConfig config, PlatformDetect.Detected detected, Supervisor.Status status) {
        JsonObject frame = new JsonObject();
        frame.addProperty("type", "hello");
        frame.addProperty("platform", detected.platform().id());
        frame.addProperty("mc_version", detected.mcVersion());
        frame.addProperty("wrapper_version", WrapperConfig.VERSION);
        frame.addProperty("server_dir", config.serverDir().toString());
        frame.add("status", status(status));
        return frame;
    }

    static JsonObject statusFrame(Supervisor.Status status) {
        JsonObject frame = status(status);
        frame.addProperty("type", "status");
        return frame;
    }

    static JsonObject console(String line) {
        JsonObject frame = new JsonObject();
        frame.addProperty("type", "console");
        frame.addProperty("line", line);
        return frame;
    }

    static JsonObject ok(long id, JsonElement data) {
        JsonObject frame = reply(id);
        frame.addProperty("ok", true);
        frame.add("data", data == null ? new JsonObject() : data);
        return frame;
    }

    static JsonObject failed(long id, String error) {
        JsonObject frame = reply(id);
        frame.addProperty("ok", false);
        frame.addProperty("error", error);
        return frame;
    }

    private static JsonObject reply(long id) {
        JsonObject frame = new JsonObject();
        frame.addProperty("type", "reply");
        frame.addProperty("id", id);
        return frame;
    }

    private static JsonObject status(Supervisor.Status status) {
        JsonObject json = new JsonObject();
        json.addProperty("running", status.running());
        json.addProperty("ready", status.ready());
        json.addProperty("uptime_secs", status.uptimeSeconds());
        if (status.exitCode() != null) {
            json.addProperty("exit_code", status.exitCode());
        }
        return json;
    }
}
