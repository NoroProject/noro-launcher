package dev.noro.agent.wrapper;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.time.Instant;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.util.Comparator;
import java.util.Locale;
import java.util.stream.Stream;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;
import java.util.zip.ZipOutputStream;
import org.slf4j.Logger;

/**
 * Снимки серверной директории.
 *
 * <p>Zip рядом с сервером, а не в облаке: цель — откатить неудачную правку
 * конфига или установку мода, а не пережить потерю машины. Настоящий бэкап это
 * не заменяет и заменять не пытается.
 */
final class ServerBackups {

    /** Каталог снимков. Сам в снимок не попадает — иначе он удваивался бы. */
    static final String DIR = "noro-backups";

    private static final DateTimeFormatter STAMP =
            DateTimeFormatter.ofPattern("yyyyMMdd-HHmmss", Locale.ROOT).withZone(ZoneOffset.UTC);

    private final ServerPaths paths;
    private final Logger log;

    ServerBackups(ServerPaths paths, Logger log) {
        this.paths = paths;
        this.log = log;
    }

    JsonElement create(String label) throws IOException {
        Path dir = Files.createDirectories(paths.resolve(DIR));
        String name = STAMP.format(Instant.now()) + (label.isBlank() ? "" : "-" + safe(label)) + ".zip";
        Path archive = dir.resolve(name);

        Path root = paths.root();
        try (OutputStream out = Files.newOutputStream(archive);
                ZipOutputStream zip = new ZipOutputStream(out)) {
            Files.walkFileTree(root, new BackupPacker(root, paths.configFile(), zip));
        }
        log.info("Backup {} is {} bytes", name, Files.size(archive));
        return describe(archive);
    }

    JsonElement list() throws IOException {
        Path dir = paths.resolve(DIR);
        JsonArray entries = new JsonArray();
        if (Files.isDirectory(dir)) {
            try (Stream<Path> files = Files.list(dir)) {
                // Свежие сверху: откатывают обычно последнее изменение.
                for (Path file : files.filter(f -> f.getFileName().toString().endsWith(".zip"))
                        .sorted(Comparator.comparing((Path f) -> f.getFileName().toString()).reversed())
                        .toList()) {
                    entries.add(describe(file));
                }
            }
        }
        JsonObject result = new JsonObject();
        result.add("backups", entries);
        return result;
    }

    /**
     * Разворачивает архив поверх текущего состояния.
     *
     * <p>Не «удалить всё и распаковать»: снести серверную директорию под живым
     * процессом — верный способ получить полусломанный сервер вместо отката.
     * Останавливать сервер должен администратор, и админка об этом предупреждает.
     */
    JsonElement restore(String name) throws IOException {
        Path archive = backupFile(name);
        int restored = 0;
        int skipped = 0;
        try (ZipInputStream zip = new ZipInputStream(Files.newInputStream(archive))) {
            ZipEntry entry;
            while ((entry = zip.getNextEntry()) != null) {
                if (entry.isDirectory()) {
                    continue;
                }
                // Имя внутри архива тоже чужое: путь проверяем той же песочницей.
                Path target;
                try {
                    target = paths.resolve(entry.getName());
                } catch (IOException e) {
                    // Одна подозрительная запись не повод бросить откат на
                    // половине — это было бы хуже, чем её пропустить.
                    log.warn("Skipping {} from the backup: {}", entry.getName(), e.getMessage());
                    skipped++;
                    continue;
                }
                Files.createDirectories(target.getParent());
                Files.copy(zip, target, StandardCopyOption.REPLACE_EXISTING);
                restored++;
            }
        }
        log.info("Restored {} files from {} ({} skipped)", restored, name, skipped);
        JsonObject result = new JsonObject();
        result.addProperty("restored", restored);
        result.addProperty("skipped", skipped);
        result.addProperty("restart_required", true);
        return result;
    }

    JsonElement delete(String name) throws IOException {
        Files.delete(backupFile(name));
        return null;
    }

    private Path backupFile(String name) throws IOException {
        Path archive = paths.resolve(DIR + "/" + safe(name));
        if (!Files.isRegularFile(archive)) {
            throw new IOException("no such backup: " + name);
        }
        return archive;
    }

    private JsonObject describe(Path archive) throws IOException {
        JsonObject json = new JsonObject();
        json.addProperty("name", archive.getFileName().toString());
        json.addProperty("size", Files.size(archive));
        json.addProperty("created", Files.getLastModifiedTime(archive).toMillis());
        return json;
    }

    /** Имя снимка задаёт человек — в путь оно попадать не должно. */
    private static String safe(String raw) {
        return raw.replaceAll("[^A-Za-z0-9._-]", "-");
    }
}
