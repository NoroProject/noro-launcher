package dev.noro.agent.wrapper;

import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.HeartbeatTask;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.ServerStatus;
import java.nio.file.Path;
import java.time.Duration;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * ServerWrapper: установщик и супервизор.
 *
 * <p>Ставит агент под платформу, прокидывает authlib-injector, держит секрет и
 * отчитывается о живости, пока сервер грузится. Авторизацию он не делает и
 * делать не должен — она на мастере, и там ей место.
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

        // Сервер грузится минуты, а мастер считает его мёртвым через 90 секунд.
        // Без этого heartbeat карточка в лаунчере всё это время серая.
        HeartbeatTask boot = bootHeartbeat(config, detected);
        boot.start();

        int code = new ServerProcess(config, javaagent, LOG).run(boot::close);
        boot.close();
        System.exit(code);
    }

    private static HeartbeatTask bootHeartbeat(WrapperConfig config, PlatformDetect.Detected detected) {
        AgentConfig agentConfig =
                new AgentConfig(config.masterUrl(), config.secret(), Duration.ofSeconds(30), true);
        ServerStatus starting = new ServerStatus() {
            @Override
            public int online() {
                return 0;
            }

            @Override
            public int maxPlayers() {
                return 0;
            }

            @Override
            public String version() {
                return detected.platform().id() + " " + detected.mcVersion() + " (starting)";
            }
        };
        return new HeartbeatTask(new MasterClient(agentConfig), starting, agentConfig, LOG);
    }
}
