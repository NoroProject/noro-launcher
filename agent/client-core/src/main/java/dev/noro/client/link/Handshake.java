package dev.noro.client.link;

import com.google.gson.Gson;
import dev.noro.client.NoroCore;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Handshake file the launcher drops into the game directory on launch.
 *
 * <p>The mod only knows its own gameDir and shouldn't have to guess where the
 * launcher is installed, so the port and key sit next to the game. No file means
 * the game was started outside the launcher and there is simply no panel.
 */
public record Handshake(int port, String key, int protocol) {

    /** Shared with the launcher, see {@code mod_link::HANDSHAKE_FILE}. */
    public static final String FILE = "noro-bridge.json";

    /** Contract version this mod speaks. */
    public static final int PROTOCOL = 1;

    private static final Gson GSON = new Gson();

    /** {@code null} when there is no handshake — the ordinary case, not an error. */
    public static Handshake read(Path gameDir) {
        Path file = gameDir.resolve(FILE);
        if (!Files.isReadable(file)) {
            return null;
        }
        try {
            Handshake handshake = GSON.fromJson(Files.readString(file), Handshake.class);
            if (handshake == null || handshake.key == null || handshake.key.isEmpty()) {
                return null;
            }
            return handshake;
        } catch (Exception e) {
            NoroCore.LOG.warn("could not read handshake: {}", e.toString());
            return null;
        }
    }
}
