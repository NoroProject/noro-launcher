package dev.noro.client.link;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * Конверт кадра: {@code {"type": …, "data": {…}}}.
 *
 * <p>Тут нет ни одного имени кадра. Их знают функции — конверт лишь
 * подписывает, что внутри, и разбирает `data` в тот класс, который функция
 * попросила. Иначе транспорт пришлось бы править на каждую новую возможность
 * мода, и «вспомогательный» превратился бы в «дела и немного всего».
 */
public final class Frames {

    private static final Gson GSON = new Gson();

    private Frames() {}

    /** Имя кадра. {@code null} — это не наш конверт. */
    public static String type(JsonObject envelope) {
        JsonElement type = envelope == null ? null : envelope.get("type");
        return type == null || type.isJsonNull() ? null : type.getAsString();
    }

    public static JsonObject envelope(String text) {
        try {
            return GSON.fromJson(text, JsonObject.class);
        } catch (Exception e) {
            return null;
        }
    }

    /** Содержимое кадра. Кадр без полей — обычное дело, отдаём пустой объект. */
    public static <T> T data(JsonObject envelope, Class<T> target) {
        JsonElement data = envelope.get("data");
        return GSON.fromJson(data == null || data.isJsonNull() ? new JsonObject() : data, target);
    }

    /**
     * Кадр наружу. Без полей — {@code {"type": …}} без {@code data}: половина
     * намерений именно такие, и лишний пустой объект та сторона не ждёт.
     */
    public static String write(Object frame) {
        JsonObject root = new JsonObject();
        root.addProperty("type", frame.getClass().getSimpleName());
        JsonElement data = GSON.toJsonTree(frame);
        if (data.isJsonObject() && !data.getAsJsonObject().isEmpty()) {
            root.add("data", data);
        }
        return GSON.toJson(root);
    }
}
