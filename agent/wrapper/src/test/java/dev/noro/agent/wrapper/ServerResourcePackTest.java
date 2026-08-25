package dev.noro.agent.wrapper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.slf4j.LoggerFactory;

/** Правка `server.properties`: две строки, и ни одной чужой. */
class ServerResourcePackTest {

    private static final org.slf4j.Logger LOG = LoggerFactory.getLogger(ServerResourcePackTest.class);

    @TempDir
    Path dir;

    private List<String> apply(String before, String url, String sha1) throws Exception {
        Files.writeString(dir.resolve("server.properties"), before);
        ServerResourcePack.apply(dir, url, sha1, LOG);
        return Files.readAllLines(dir.resolve("server.properties"));
    }

    /** Существующие строки переписываются на месте, порядок сохраняется. */
    @Test
    void replacesExistingLinesInPlace() throws Exception {
        List<String> out = apply(
                "motd=Noro\nresource-pack=http://old\nresource-pack-sha1=old\nmax-players=20\n",
                "http://new", "abc");

        assertEquals(
                List.of("motd=Noro", "resource-pack=http://new", "resource-pack-sha1=abc", "max-players=20"),
                out);
    }

    /** Чужие настройки и комментарии не трогаются: файл не наш. */
    @Test
    void keepsEverythingElseIntact() throws Exception {
        List<String> out = apply("#Minecraft server properties\nlevel-name=world\n", "http://p", "s");

        assertTrue(out.contains("#Minecraft server properties"));
        assertTrue(out.contains("level-name=world"));
        assertTrue(out.contains("resource-pack=http://p"));
    }

    /** Пак отключили — строки очищаются, а не остаются ссылкой в никуда. */
    @Test
    void clearsTheLinesWhenThePackIsGone() throws Exception {
        List<String> out = apply("resource-pack=http://old\nresource-pack-sha1=old\n", "", "");

        assertEquals(List.of("resource-pack=", "resource-pack-sha1="), out);
    }

    /** Нет файла — нет и правки: сервер ещё не создал конфиг. */
    @Test
    void survivesAMissingFile() {
        ServerResourcePack.apply(dir.resolve("nope"), "http://p", "s", LOG);
    }
}
