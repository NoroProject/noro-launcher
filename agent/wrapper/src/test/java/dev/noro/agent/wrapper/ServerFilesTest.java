package dev.noro.agent.wrapper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class ServerFilesTest {

    @TempDir
    Path tmp;

    private Path serverDir;
    private ServerFiles files;

    @BeforeEach
    void setUp() throws IOException {
        serverDir = Files.createDirectories(tmp.resolve("server"));
        Path config = Files.writeString(serverDir.resolve("noro-wrapper.properties"), "secret=x");
        files = new ServerFiles(new ServerPaths(serverDir, config));
    }

    @Test
    void listsDirectoriesFirst() throws IOException {
        Files.writeString(serverDir.resolve("server.properties"), "motd=hi");
        Files.createDirectories(serverDir.resolve("plugins"));

        JsonObject listing = files.list(".").getAsJsonObject();
        var entries = listing.getAsJsonArray("entries");
        assertEquals("plugins", entries.get(0).getAsJsonObject().get("name").getAsString());
        assertTrue(entries.get(0).getAsJsonObject().get("dir").getAsBoolean());
    }

    @Test
    void writesThenReadsBack() throws IOException {
        files.write("config/mod.toml", "enabled = true\n");
        JsonObject read = files.read("config/mod.toml").getAsJsonObject();
        assertEquals("enabled = true\n", read.get("content").getAsString());
        // Временный файл после атомарной замены остаться не должен.
        assertFalse(Files.exists(serverDir.resolve("config/mod.toml.noro-tmp")));
    }

    @Test
    void refusesToReadBinary() throws IOException {
        Files.write(serverDir.resolve("mod.jar"), new byte[] {(byte) 0xFF, (byte) 0xFE, 0x00, 0x01});
        IOException error = assertThrows(IOException.class, () -> files.read("mod.jar"));
        assertTrue(error.getMessage().contains("not a text file"), error.getMessage());
    }

    @Test
    void deletesDirectoryTree() throws IOException {
        files.write("config/deep/a.txt", "a");
        files.delete("config");
        assertFalse(Files.exists(serverDir.resolve("config")));
    }

    @Test
    void refusesToDeleteTheServerDirectory() {
        IOException error = assertThrows(IOException.class, () -> files.delete("."));
        assertTrue(error.getMessage().contains("server directory itself"), error.getMessage());
    }

    @Test
    void escapeAttemptsNeverTouchTheDisk() throws IOException {
        Path victim = Files.writeString(tmp.resolve("victim.txt"), "still here");
        assertThrows(IOException.class, () -> files.delete("../victim.txt"));
        assertThrows(IOException.class, () -> files.write("../victim.txt", "overwritten"));
        assertEquals("still here", Files.readString(victim));
    }
}
