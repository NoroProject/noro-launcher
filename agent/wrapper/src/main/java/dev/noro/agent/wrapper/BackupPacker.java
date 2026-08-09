package dev.noro.agent.wrapper;

import java.io.IOException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

/** Обход серверной директории в zip с пропуском служебных каталогов. */
final class BackupPacker extends SimpleFileVisitor<Path> {

    /** Что не архивируем: логи, кеши и сам каталог снимков. */
    private static final List<String> SKIP =
            List.of(ServerBackups.DIR, "logs", "crash-reports", "cache", ".git");

    private final Path root;
    private final ZipOutputStream zip;
    /** Конфиг враппера: в нём секрет игрового сервера открытым текстом. */
    private final Path excluded;

    BackupPacker(Path root, Path excluded, ZipOutputStream zip) {
        this.root = root;
        this.excluded = excluded;
        this.zip = zip;
    }

    @Override
    public FileVisitResult preVisitDirectory(Path dir, BasicFileAttributes attrs) {
        boolean skip = !dir.equals(root) && SKIP.contains(dir.getFileName().toString());
        return skip ? FileVisitResult.SKIP_SUBTREE : FileVisitResult.CONTINUE;
    }

    @Override
    public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) throws IOException {
        if (file.equals(excluded)) {
            return FileVisitResult.CONTINUE;
        }
        zip.putNextEntry(new ZipEntry(root.relativize(file).toString()));
        Files.copy(file, zip);
        zip.closeEntry();
        return FileVisitResult.CONTINUE;
    }

    @Override
    public FileVisitResult visitFileFailed(Path file, IOException error) {
        // Файл под записью сервером — пропускаем: снимок важнее полноты.
        return FileVisitResult.CONTINUE;
    }
}
