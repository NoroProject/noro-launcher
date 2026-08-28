package dev.noro.client.link;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * Frame envelope: {@code {"type": …, "data": {…}}}.
 *
 * <p>Deliberately contains no frame names. Features know those; the envelope
 * only labels what's inside and parses {@code data} into whatever class the
 * feature asked for. Otherwise the transport would need editing for every new
 * capability.
 */
public final class Frames {

    private static final Gson GSON = new Gson();

    private Frames() {}

    /** Frame name, or {@code null} when this isn't our envelope. */
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

    /** A frame with no fields is common, so a missing {@code data} parses as an empty object. */
    public static <T> T data(JsonObject envelope, Class<T> target) {
        JsonElement data = envelope.get("data");
        return GSON.fromJson(data == null || data.isJsonNull() ? new JsonObject() : data, target);
    }

    /**
     * Outbound frame. A field-less one is written as {@code {"type": …}} with no
     * {@code data} — the other side doesn't expect a stray empty object.
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
