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
import org.slf4j.Logger;

/**
 * Запуск и надзор за процессом сервера.
 *
 * <p>Секрет уходит дочернему процессу переменной окружения — так он существует
 * ровно в одном месте на диске, в конфиге враппера, а не дублируется в конфиг
 * агента внутри {@code plugins/} или {@code mods/}.
 */
public final class ServerProcess {

    /** Ванильное сообщение о готовности; его печатают все три платформы. */
    private static final String READY_MARKER = "Done (";

    private final WrapperConfig config;
    private final String javaagentArg;
    private final Logger log;

    public ServerProcess(WrapperConfig config, String javaagentArg, Logger log) {
        this.config = config;
        this.javaagentArg = javaagentArg;
        this.log = log;
    }

    /**
     * @param onReady вызывается, когда сервер сообщил о готовности
     * @return код возврата процесса
     */
    public int run(Runnable onReady) throws IOException, InterruptedException {
        ProcessBuilder builder = new ProcessBuilder(command());
        builder.directory(config.serverDir().toFile());
        builder.redirectErrorStream(true);
        builder.environment().put(AgentConfig.ENV_URL, config.masterUrl());
        builder.environment().put(AgentConfig.ENV_SECRET, config.secret());

        log.info("Starting: {}", String.join(" ", command()));
        Process process = builder.start();

        Thread stop = new Thread(() -> shutdown(process));
        Runtime.getRuntime().addShutdownHook(stop);

        pipeConsoleInto(process);
        readOutput(process, onReady);

        int code = process.waitFor();
        // Хук уже отработал, если гасили нас; снимаем его, чтобы не звать дважды.
        Runtime.getRuntime().removeShutdownHook(stop);
        log.info("Server exited with code {}", code);
        return code;
    }

    private List<String> command() {
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

    /** Консоль администратора должна доходить до сервера, иначе это не супервизор. */
    private void pipeConsoleInto(Process process) {
        Thread pump = new Thread(() -> {
            try (var console = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
                    Writer toServer =
                            new OutputStreamWriter(process.getOutputStream(), StandardCharsets.UTF_8)) {
                String line;
                while ((line = console.readLine()) != null) {
                    toServer.write(line);
                    toServer.write('\n');
                    toServer.flush();
                }
            } catch (IOException ignored) {
                // Сервер закрыл поток — читать больше некуда, выходим молча.
            }
        }, "noro-wrapper-console");
        pump.setDaemon(true);
        pump.start();
    }

    private void readOutput(Process process, Runnable onReady) throws IOException {
        boolean ready = false;
        try (var out = new BufferedReader(
                new InputStreamReader(process.getInputStream(), StandardCharsets.UTF_8))) {
            String line;
            while ((line = out.readLine()) != null) {
                System.out.println(line);
                if (!ready && line.contains(READY_MARKER)) {
                    ready = true;
                    // Дальше heartbeat шлёт агент изнутри сервера, и он знает
                    // настоящий онлайн — враппер тут больше не нужен.
                    onReady.run();
                }
            }
        }
    }

    /** Останавливаем сервер его же командой, чтобы мир успел сохраниться. */
    private void shutdown(Process process) {
        if (!process.isAlive()) {
            return;
        }
        try (Writer toServer = new OutputStreamWriter(process.getOutputStream(), StandardCharsets.UTF_8)) {
            toServer.write("stop\n");
            toServer.flush();
        } catch (IOException ignored) {
            // Поток уже закрыт — ниже добьём принудительно.
        }
        try {
            if (!process.waitFor(60, TimeUnit.SECONDS)) {
                process.destroyForcibly();
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            process.destroyForcibly();
        }
    }
}
