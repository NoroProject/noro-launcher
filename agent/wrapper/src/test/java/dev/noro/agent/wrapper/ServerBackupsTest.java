package dev.noro.agent.wrapper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.slf4j.LoggerFactory;

class ServerBackupsTest {

    @TempDir
    Path tmp;

    private Path serverDir;
    private ServerBackups backups;

    @BeforeEach
    void setUp() throws IOException {
        serverDir = Files.createDirectories(tmp.resolve("server"));
        Path config = Files.writeString(serverDir.resolve("noro-wrapper.properties"), "secret=x");
        backups = new ServerBackups(
                new ServerPaths(serverDir, config), LoggerFactory.getLogger("test"));
    }

    @Test
    void packsTheServerButNotTheBackupsOrLogs() throws IOException {
        Files.writeString(serverDir.resolve("server.properties"), "motd=old");
        Files.createDirectories(serverDir.resolve("logs"));
        Files.writeString(serverDir.resolve("logs/latest.log"), "noise");

        JsonObject made = backups.create("before-update").getAsJsonObject();
        Path archive = serverDir.resolve(ServerBackups.DIR).resolve(made.get("name").getAsString());
        assertTrue(Files.isRegularFile(archive));

        try (ZipFile zip = new ZipFile(archive.toFile())) {
            assertTrue(zip.getEntry("server.properties") != null);
            // Логи и сам каталог снимков внутрь попадать не должны.
            assertTrue(zip.getEntry("logs/latest.log") == null);
            assertTrue(zip.stream().map(ZipEntry::getName).noneMatch(n -> n.startsWith(ServerBackups.DIR)));
            // Конфиг враппера — не состояние сервера, а секрет: в архив не идёт.
            assertTrue(zip.getEntry("noro-wrapper.properties") == null);
        }
    }

    @Test
    void restoreBringsTheOldContentBack() throws IOException {
        Files.writeString(serverDir.resolve("server.properties"), "motd=old");
        String name = backups.create("").getAsJsonObject().get("name").getAsString();

        Files.writeString(serverDir.resolve("server.properties"), "motd=broken");
        JsonObject result = backups.restore(name).getAsJsonObject();

        assertEquals("motd=old", Files.readString(serverDir.resolve("server.properties")));
        assertTrue(result.get("restored").getAsInt() >= 1);
        assertTrue(result.get("restart_required").getAsBoolean());
    }

    @Test
    void listsNewestFirstAndDeletes() throws IOException {
        Files.writeString(serverDir.resolve("a.txt"), "a");
        String first = backups.create("aaa").getAsJsonObject().get("name").getAsString();
        String second = backups.create("bbb").getAsJsonObject().get("name").getAsString();

        var listed = backups.list().getAsJsonObject().getAsJsonArray("backups");
        assertEquals(2, listed.size());
        // Имя начинается с метки времени, так что обратный порядок имён — это
        // «свежие сверху». Секундной точности здесь хватает: имена различаются
        // суффиксом, если совпал момент.
        assertTrue(listed.get(0).getAsJsonObject().get("name").getAsString().compareTo(
                        listed.get(1).getAsJsonObject().get("name").getAsString())
                >= 0);

        backups.delete(first);
        assertFalse(Files.exists(serverDir.resolve(ServerBackups.DIR).resolve(first)));
        assertTrue(Files.exists(serverDir.resolve(ServerBackups.DIR).resolve(second)));
    }

    @Test
    void backupNameCannotEscape() {
        // Имя задаёт человек: любой разделитель пути в нём обезвреживается.
        assertThrows(IOException.class, () -> backups.delete("../../server.properties"));
    }
}
