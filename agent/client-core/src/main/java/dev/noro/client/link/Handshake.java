package dev.noro.client.link;

import com.google.gson.Gson;
import dev.noro.client.NoroCore;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Файл рукопожатия, который лаунчер кладёт в каталог игры на запуск.
 *
 * <p>Мод знает только свой gameDir и не должен угадывать, где установлен
 * лаунчер, — поэтому порт и ключ лежат рядом с игрой. Файла нет — значит игру
 * запустили не через лаунчер, и панели просто не будет.
 */
public record Handshake(int port, String key, int protocol) {

    /** Имя файла — общее с лаунчером, см. {@code mod_link::HANDSHAKE_FILE}. */
    public static final String FILE = "noro-bridge.json";

    /** Версия договора, которую понимает этот мод. */
    public static final int PROTOCOL = 1;

    private static final Gson GSON = new Gson();

    /** {@code null} — рукопожатия нет; это обычный случай, а не ошибка. */
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
            NoroCore.LOG.warn("рукопожатие не прочиталось: {}", e.toString());
            return null;
        }
    }
}
