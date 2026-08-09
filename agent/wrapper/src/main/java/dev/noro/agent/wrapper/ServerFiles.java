package dev.noro.agent.wrapper;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.io.IOException;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Comparator;
import java.util.List;
import java.util.stream.Stream;

/** Файловые операции в серверной директории: список, чтение, запись, удаление. */
final class ServerFiles {

    /** Потолок текстового файла. Конфиги мельче на порядки; всё крупнее — не конфиг. */
    private static final long MAX_TEXT_BYTES = 1024 * 1024;

    private final ServerPaths paths;

    ServerFiles(ServerPaths paths) {
        this.paths = paths;
    }

    JsonElement list(String relative) throws IOException {
        Path dir = paths.resolve(relative);
        if (!Files.isDirectory(dir)) {
            throw new IOException("not a directory: " + relative);
        }
        JsonArray entries = new JsonArray();
        try (Stream<Path> children = Files.list(dir)) {
            List<Path> sorted = children.sorted(ServerFiles::directoriesFirst).toList();
            for (Path child : sorted) {
                entries.add(describe(child));
            }
        }
        JsonObject result = new JsonObject();
        result.addProperty("path", paths.relativize(dir));
        result.add("entries", entries);
        return result;
    }

    JsonElement read(String relative) throws IOException {
        Path file = paths.resolve(relative);
        if (!Files.isRegularFile(file)) {
            throw new IOException("not a file: " + relative);
        }
        long size = Files.size(file);
        if (size > MAX_TEXT_BYTES) {
            throw new IOException("file is %d bytes, too large to edit as text".formatted(size));
        }
        JsonObject result = new JsonObject();
        result.addProperty("path", paths.relativize(file));
        result.addProperty("content", text(file));
        return result;
    }

    JsonElement write(String relative, String content) throws IOException {
        Path file = paths.resolve(relative);
        Files.createDirectories(file.getParent());
        // Через временный файл: оборванная запись не должна оставить конфиг
        // наполовину переписанным — сервер такой не переживёт.
        Path tmp = file.resolveSibling(file.getFileName() + ".noro-tmp");
        Files.writeString(tmp, content, StandardCharsets.UTF_8);
        Files.move(tmp, file, StandardCopyOption.REPLACE_EXISTING);

        JsonObject result = new JsonObject();
        result.addProperty("path", paths.relativize(file));
        result.addProperty("size", Files.size(file));
        return result;
    }

    JsonElement delete(String relative) throws IOException {
        Path target = paths.resolve(relative);
        if (target.equals(paths.root())) {
            throw new IOException("refusing to delete the server directory itself");
        }
        if (!Files.exists(target)) {
            throw new IOException("no such path: " + relative);
        }
        if (Files.isDirectory(target)) {
            try (Stream<Path> tree = Files.walk(target)) {
                // Снизу вверх: каталог удаляется только пустым.
                for (Path path : tree.sorted(Comparator.reverseOrder()).toList()) {
                    Files.delete(path);
                }
            }
        } else {
            Files.delete(target);
        }
        return null;
    }

    JsonElement mkdir(String relative) throws IOException {
        Files.createDirectories(paths.resolve(relative));
        return null;
    }

    private static String text(Path file) throws IOException {
        try {
            return StandardCharsets.UTF_8
                    .newDecoder()
                    .decode(java.nio.ByteBuffer.wrap(Files.readAllBytes(file)))
                    .toString();
        } catch (CharacterCodingException e) {
            // Читать jar как текст бессмысленно, а отдать мусор — хуже отказа.
            throw new IOException("not a text file: " + file.getFileName());
        }
    }

    private JsonObject describe(Path path) throws IOException {
        boolean dir = Files.isDirectory(path);
        JsonObject entry = new JsonObject();
        entry.addProperty("name", path.getFileName().toString());
        entry.addProperty("path", paths.relativize(path));
        entry.addProperty("dir", dir);
        entry.addProperty("size", dir ? 0 : Files.size(path));
        entry.addProperty("modified", Files.getLastModifiedTime(path).toMillis());
        return entry;
    }

    /** Каталоги сверху, затем по имени — так же, как в любом файловом менеджере. */
    private static int directoriesFirst(Path a, Path b) {
        boolean da = Files.isDirectory(a);
        if (da != Files.isDirectory(b)) {
            return da ? -1 : 1;
        }
        return a.getFileName().toString().compareToIgnoreCase(b.getFileName().toString());
    }
}
