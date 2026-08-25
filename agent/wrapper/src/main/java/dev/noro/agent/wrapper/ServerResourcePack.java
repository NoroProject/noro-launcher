package dev.noro.agent.wrapper;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import org.slf4j.Logger;

/**
 * Пак плашек в {@code server.properties}.
 *
 * <p>Агент выдаёт пак сам, но только тем, кто уже вошёл и чей клиент это умеет.
 * Строка в конфиге закрывает остальных: зашедшего мимо лаунчера и версии старше
 * 1.20.3, где пак применяется только при подключении.
 *
 * <p>Правит ровно две строки и ничего больше: файл принадлежит владельцу
 * сервера, и переписывать его целиком — верный способ однажды стереть чужую
 * настройку. Порядок строк сохраняется, комментарии тоже.
 */
final class ServerResourcePack {

    private static final String URL_KEY = "resource-pack";
    private static final String SHA_KEY = "resource-pack-sha1";

    private ServerResourcePack() {}

    /**
     * Прописать адрес и контрольную сумму.
     *
     * <p>Пустой адрес стирает обе строки: пак отключили на мастере, и оставлять
     * ссылку на исчезнувший файл нельзя — клиент будет ругаться при каждом входе.
     */
    static void apply(Path serverDir, String url, String sha1, Logger log) {
        Path file = serverDir.resolve("server.properties");
        if (!Files.isRegularFile(file)) {
            // Первый запуск: сервер ещё не создал конфиг. Пропишем в следующий,
            // а пока пак доедет выдачей агента.
            return;
        }
        try {
            List<String> lines = Files.readAllLines(file, StandardCharsets.UTF_8);
            List<String> updated = rewrite(lines, url, sha1);
            if (!updated.equals(lines)) {
                Files.write(file, updated, StandardCharsets.UTF_8);
                log.info("Resource pack in server.properties: {}", url.isEmpty() ? "cleared" : url);
            }
        } catch (IOException e) {
            // Не повод не запускать сервер: без строки пак просто не доедет до
            // тех, кто зашёл мимо лаунчера.
            log.warn("Cannot update server.properties: {}", e.toString());
        }
    }

    private static List<String> rewrite(List<String> lines, String url, String sha1) {
        List<String> out = new ArrayList<>(lines.size() + 2);
        boolean hadUrl = false;
        boolean hadSha = false;
        for (String line : lines) {
            if (line.startsWith(URL_KEY + "=")) {
                out.add(URL_KEY + "=" + url);
                hadUrl = true;
            } else if (line.startsWith(SHA_KEY + "=")) {
                out.add(SHA_KEY + "=" + sha1);
                hadSha = true;
            } else {
                out.add(line);
            }
        }
        if (!hadUrl) {
            out.add(URL_KEY + "=" + url);
        }
        if (!hadSha) {
            out.add(SHA_KEY + "=" + sha1);
        }
        return out;
    }
}
