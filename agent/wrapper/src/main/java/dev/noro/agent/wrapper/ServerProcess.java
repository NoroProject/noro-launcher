package dev.noro.agent.wrapper;

import dev.noro.agent.core.AgentConfig;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import java.util.function.Consumer;
import org.slf4j.Logger;

/**
 * Один запуск сервера: процесс, его ввод и вывод.
 *
 * <p>Раньше здесь же был и жизненный цикл — метод {@code run} блокировался до
 * выхода процесса, и враппер умирал вместе с сервером. Теперь это только ручка
 * над процессом, а решения «когда стартовать и надо ли поднимать снова»
 * принимает {@link Supervisor}: иначе на погашенном сервере управлять нечем.
 *
 * <p>Секрет уходит дочернему процессу переменной окружения — так он существует
 * ровно в одном месте на диске, в конфиге враппера, а не дублируется в конфиг
 * агента внутри {@code plugins/} или {@code mods/}.
 */
public final class ServerProcess {

    /** Ванильное сообщение о готовности; его печатают все три платформы. */
    private static final String READY_MARKER = "Done (";

    private final Process process;
    private final Writer input;
    private final long startedAt = System.currentTimeMillis();
    private volatile boolean ready;

    private ServerProcess(Process process) {
        this.process = process;
        this.input = new OutputStreamWriter(process.getOutputStream(), StandardCharsets.UTF_8);
    }

    /**
     * @param onLine   каждая строка вывода — в консоль машины и в канал мастера
     * @param onReady  сервер сообщил о готовности принимать игроков
     */
    public static ServerProcess start(
            WrapperConfig config, String javaagentArg, Logger log, Consumer<String> onLine, Runnable onReady)
            throws IOException {
        List<String> command = command(config, javaagentArg);
        ProcessBuilder builder = new ProcessBuilder(command);
        builder.directory(config.serverDir().toFile());
        builder.redirectErrorStream(true);
        builder.environment().put(AgentConfig.ENV_URL, config.masterUrl());
        builder.environment().put(AgentConfig.ENV_SECRET, config.secret());

        log.info("Starting: {}", String.join(" ", command));
        ServerProcess server = new ServerProcess(builder.start());
        server.pumpOutput(onLine, onReady);
        return server;
    }

    private static List<String> command(WrapperConfig config, String javaagentArg) {
        List<String> command = new ArrayList<>();
        command.add(config.javaBin());
        command.addAll(config.jvmArgs());
        command.add(javaagentArg);
        // NeoForge стартует не из jar'а, а из @-файла аргументов, поэтому такое
        // значение пробрасываем как есть.
        if (config.serverJar().startsWith("@")) {
            command.add(config.serverJar());
        } else {
            command.add("-jar");
            command.add(config.serverJar());
        }
        command.addAll(config.serverArgs());
        return command;
    }

    private void pumpOutput(Consumer<String> onLine, Runnable onReady) {
        Thread pump = new Thread(
                () -> {
                    try (var out = new BufferedReader(
                            new InputStreamReader(process.getInputStream(), StandardCharsets.UTF_8))) {
                        String line;
                        while ((line = out.readLine()) != null) {
                            System.out.println(line);
                            onLine.accept(line);
                            if (!ready && line.contains(READY_MARKER)) {
                                ready = true;
                                onReady.run();
                            }
                        }
                    } catch (IOException ignored) {
                        // Процесс закрыл поток — читать больше нечего.
                    }
                },
                "noro-server-output");
        pump.setDaemon(true);
        pump.start();
    }

    /** Строка в консоль сервера. Молча игнорируется, если процесс уже мёртв. */
    public synchronized void send(String line) {
        if (!process.isAlive()) {
            return;
        }
        try {
            input.write(line);
            input.write('\n');
            input.flush();
        } catch (IOException ignored) {
            // Поток закрыт — сервер уже уходит, писать некуда.
        }
    }

    public boolean isAlive() {
        return process.isAlive();
    }

    public boolean ready() {
        return ready;
    }

    public long uptimeSeconds() {
        return (System.currentTimeMillis() - startedAt) / 1000;
    }

    /** Останавливаем сервер его же командой, чтобы мир успел сохраниться. */
    public int stopGracefully(long timeoutSeconds) throws InterruptedException {
        if (!process.isAlive()) {
            return process.exitValue();
        }
        send("stop");
        if (!process.waitFor(timeoutSeconds, TimeUnit.SECONDS)) {
            process.destroyForcibly();
        }
        return process.waitFor();
    }

    public int kill() throws InterruptedException {
        process.destroyForcibly();
        return process.waitFor();
    }

    public int waitFor() throws InterruptedException {
        return process.waitFor();
    }
}
