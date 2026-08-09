package dev.noro.agent.wrapper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/** Песочница путей: наружу серверной директории выйти нельзя. */
class ServerPathsTest {

    @TempDir
    Path tmp;

    private Path serverDir;
    private Path configFile;
    private ServerPaths paths;

    @BeforeEach
    void setUp() throws IOException {
        serverDir = Files.createDirectories(tmp.resolve("server"));
        configFile = Files.writeString(serverDir.resolve("noro-wrapper.properties"), "secret=x");
        paths = new ServerPaths(serverDir, configFile);
    }

    @Test
    void resolvesInsideTheServerDirectory() throws IOException {
        Path resolved = paths.resolve("config/mod.toml");
        assertTrue(resolved.startsWith(serverDir.toRealPath()), resolved.toString());
    }

    @Test
    void rejectsDotDot() {
        IOException error = assertThrows(IOException.class, () -> paths.resolve("../outside.txt"));
        assertTrue(error.getMessage().contains("escapes"), error.getMessage());
    }

    @Test
    void rejectsAbsolutePaths() {
        assertThrows(IOException.class, () -> paths.resolve("/etc/passwd"));
    }

    @Test
    void rejectsSymlinkPointingOutside() throws IOException {
        Path outside = Files.createDirectories(tmp.resolve("outside"));
        Files.writeString(outside.resolve("loot.txt"), "secrets");
        try {
            Files.createSymbolicLink(serverDir.resolve("escape"), outside);
        } catch (UnsupportedOperationException | IOException e) {
            return; // Файловая система без симлинков — проверять нечего.
        }
        // Путь выглядит внутренним, но ведёт наружу: ловим по настоящему пути.
        assertThrows(IOException.class, () -> paths.resolve("escape/loot.txt"));
    }

    @Test
    void wrapperConfigIsNotReachable() {
        // В нём секрет игрового сервера открытым текстом.
        IOException error =
                assertThrows(IOException.class, () -> paths.resolve("noro-wrapper.properties"));
        assertTrue(error.getMessage().contains("secret"), error.getMessage());
    }

    @Test
    void relativizesBackToWhatTheAdminSees() throws IOException {
        assertEquals("mods/thing.jar", paths.relativize(paths.resolve("mods/thing.jar")));
    }
}
