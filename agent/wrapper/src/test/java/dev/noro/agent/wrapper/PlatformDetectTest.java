package dev.noro.agent.wrapper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * Определение платформы проверяется на собранных здесь же заготовках: держать
 * в тестах настоящие серверные jar'ы по полсотни мегабайт незачем, а признаки,
 * по которым идёт распознавание, воспроизводятся точно.
 */
class PlatformDetectTest {

    @TempDir
    Path serverDir;

    @Test
    void detectsPaperByVersionJson() throws IOException {
        // Ровно так устроен настоящий paperclip: version.json в корне jar'а.
        writeJar("paper-1.21.1.jar", Map.of("version.json", "{\"id\": \"1.21.1\", \"name\": \"1.21.1\"}"));

        PlatformDetect.Detected detected = PlatformDetect.detect(config("paper-1.21.1.jar", null, null));
        assertEquals(Platform.PAPER, detected.platform());
        assertEquals("1.21.1", detected.mcVersion());
        assertEquals(serverDir.resolve("plugins"), detected.platform().installDir(serverDir));
    }

    @Test
    void detectsFabricByInstallProperties() throws IOException {
        writeJar(
                "fabric-server-launch.jar",
                Map.of("install.properties", "fabric-loader-version=0.19.3\ngame-version=1.21.1\n"));

        PlatformDetect.Detected detected =
                PlatformDetect.detect(config("fabric-server-launch.jar", null, null));
        assertEquals(Platform.FABRIC, detected.platform());
        assertEquals("1.21.1", detected.mcVersion());
        assertEquals(serverDir.resolve("mods"), detected.platform().installDir(serverDir));
    }

    @Test
    void detectsNeoForgeByLibrariesTree() throws IOException {
        // NeoForge стартует не из jar'а, поэтому узнаётся только по каталогу.
        Files.createDirectories(serverDir.resolve("libraries/net/neoforged/neoforge/21.1.248"));

        PlatformDetect.Detected detected =
                PlatformDetect.detect(config("@libraries/unix_args.txt", null, null));
        assertEquals(Platform.NEOFORGE, detected.platform());
        // 21.1.x в схеме NeoForge означает Minecraft 1.21.1.
        assertEquals("1.21.1", detected.mcVersion());
    }

    @Test
    void explicitOverrideWins() throws IOException {
        writeJar("paper-1.21.1.jar", Map.of("version.json", "{\"id\": \"1.21.1\"}"));

        PlatformDetect.Detected detected =
                PlatformDetect.detect(config("paper-1.21.1.jar", "fabric", "1.21.4"));
        assertEquals(Platform.FABRIC, detected.platform());
        assertEquals("1.21.4", detected.mcVersion());
    }

    @Test
    void unknownServerAsksForExplicitSettings() throws IOException {
        writeJar("mystery.jar", Map.of("README.txt", "nothing useful here"));

        IOException error =
                assertThrows(IOException.class, () -> PlatformDetect.detect(config("mystery.jar", null, null)));
        assertEquals(true, error.getMessage().contains("noro-wrapper.properties"), error.getMessage());
    }

    private WrapperConfig config(String serverJar, String platform, String mcVersion) {
        return new WrapperConfig(
                "http://127.0.0.1:8080",
                "noroagent_" + "ab".repeat(32),
                "735fcfcd88e76de41d49ab3b6e42ac506ddf213359425b047a05d4c4a77f0715",
                serverDir,
                serverJar,
                "java",
                List.of(),
                List.of(),
                platform,
                mcVersion);
    }

    private void writeJar(String name, Map<String, String> entries) throws IOException {
        try (OutputStream out = Files.newOutputStream(serverDir.resolve(name));
                ZipOutputStream zip = new ZipOutputStream(out)) {
            for (Map.Entry<String, String> entry : entries.entrySet()) {
                zip.putNextEntry(new ZipEntry(entry.getKey()));
                zip.write(entry.getValue().getBytes(StandardCharsets.UTF_8));
                zip.closeEntry();
            }
        }
    }
}
