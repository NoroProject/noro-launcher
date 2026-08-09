package dev.noro.agent.wrapper;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * ServerWrapper: установщик и супервизор.
 *
 * <p>Ставит агент под платформу, прокидывает authlib-injector, держит секрет и
 * управляет процессом сервера по командам мастера. Авторизацию он не делает и
 * делать не должен — она на мастере, и там ей место.
 *
 * <p>Живёт дольше сервера: без этого мод, поставленный из админки, применить
 * нечем, а на упавшем сервере панель управления мертва.
 */
public final class Main {

    private static final Logger LOG = LoggerFactory.getLogger("noro-wrapper");

    public static void main(String[] args) throws Exception {
        Path configFile = Path.of(args.length > 0 ? args[0] : "noro-wrapper.properties");
        WrapperConfig config = WrapperConfig.load(configFile);

        PlatformDetect.Detected detected = PlatformDetect.detect(config);
        LOG.info("Detected {} {}", detected.platform().id(), detected.mcVersion());

        String signingKey = SigningKey.resolve(config, LOG);
        new AgentInstaller(config, signingKey, LOG).install(detected);
        // До запуска: иначе лоадер прочитает старое значение и наш обработчик
        // прав останется незамеченным до следующего рестарта.
        PermissionHandlerConfig.ensure(config, detected.platform(), LOG);

        String javaagent = new AuthlibInjector(config, LOG).jvmArg();
        Supervisor supervisor = new Supervisor(config, javaagent, LOG);

        ServerPaths paths = new ServerPaths(config.serverDir(), configFile);
        ControlOps ops = new ControlOps(
                supervisor,
                new ServerFiles(paths),
                new ServerMods(paths, LOG),
                new ServerBackups(paths, LOG));
        ControlLink link = new ControlLink(config, detected, supervisor, ops, LOG);
        link.start();

        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            link.close();
            supervisor.shutdown();
        }));

        if (config.autostart()) {
            supervisor.start();
        } else {
            LOG.info("autostart=false — waiting for a start command from the master");
        }

        pipeConsole(supervisor);
    }

    /**
     * Консоль администратора на самой машине должна доходить до сервера, иначе
     * это не супервизор. Заодно это и то, что держит процесс живым.
     */
    private static void pipeConsole(Supervisor supervisor) {
        try (var console = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8))) {
            String line;
            while ((line = console.readLine()) != null) {
                try {
                    supervisor.command(line);
                } catch (RuntimeException e) {
                    LOG.warn("{}", e.getMessage());
                }
            }
        } catch (IOException ignored) {
            // stdin закрыли — враппер запущен как служба, ждём дальше.
        }
        // Под systemd stdin закрыт сразу; спать здесь честнее, чем выйти и
        // унести с собой канал управления.
        parkForever();
    }

    private static void parkForever() {
        try {
            Thread.currentThread().join();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}
