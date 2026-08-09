package dev.noro.agent.wrapper;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Разрешение путей внутри серверной директории.
 *
 * <p>Это не защита от администратора — тот, кто может положить jar в
 * {@code plugins/} и перезапустить сервер, уже исполняет код на машине. Это
 * защита от выхода за пределы каталога: опечатка или {@code ../..} в запросе не
 * должны дотянуться до системы. Плюс одно настоящее ограничение — конфиг
 * враппера не читается и не пишется: в нём лежит секрет в открытом виде.
 */
final class ServerPaths {

    private final Path root;
    private final Path configFile;

    /**
     * Корень канонизируется сразу: на macOS {@code /var} сам по себе симлинк на
     * {@code /private/var}, и сравнение «настоящего» пути с ненастоящим корнем
     * объявляло бы побегом любой путь внутри каталога.
     */
    ServerPaths(Path root, Path configFile) throws IOException {
        this.root = real(root);
        this.configFile = real(configFile);
    }

    Path root() {
        return root;
    }

    /** Конфиг враппера: в снимок он попадать не должен — там секрет. */
    Path configFile() {
        return configFile;
    }

    /**
     * @param relative путь относительно серверной директории
     * @throws IOException если путь ведёт наружу или в запрещённый файл
     */
    Path resolve(String relative) throws IOException {
        // Абсолютный путь тоже сюда попадает: resolve вернёт его целиком, а
        // проверка ниже его и отсечёт.
        Path target = root.resolve(relative).normalize();
        if (!target.startsWith(root)) {
            throw new IOException("path escapes the server directory: " + relative);
        }

        // Симлинк наружу выглядит как путь внутри каталога — сверяем настоящий.
        // Сам корень пропускаем: его родитель выше корня по определению.
        Path parent = target.getParent();
        if (!target.equals(root) && parent != null && Files.exists(parent)) {
            Path realParent = parent.toRealPath();
            if (!realParent.startsWith(root)) {
                throw new IOException("path escapes the server directory via a link: " + relative);
            }
            target = realParent.resolve(target.getFileName());
        }
        if (Files.exists(target) && !target.toRealPath().startsWith(root)) {
            throw new IOException("path escapes the server directory via a link: " + relative);
        }

        if (target.equals(configFile)) {
            throw new IOException("the wrapper config holds the server secret and is not accessible");
        }
        return target;
    }

    /** Путь относительно корня — то, чем оперирует админка. */
    String relativize(Path path) {
        String relative = root.relativize(path).toString();
        // Сам корень относительно себя — пустая строка; админке нужен «.».
        return relative.isEmpty() ? "." : relative;
    }

    private static Path real(Path path) throws IOException {
        return Files.exists(path) ? path.toRealPath() : path.toAbsolutePath().normalize();
    }
}
