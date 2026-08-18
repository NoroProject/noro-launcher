package dev.noro.agent.core;

import java.io.IOException;
import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * Профиль игрока, сигнал жизни и каталог прав. Знает про игру ровно ничего.
 *
 * <p>Авторизация — секретом игрового сервера. {@code server_id} мастер берёт из
 * самого секрета, поэтому агент нигде не сообщает, за какой сервер отчитывается.
 * Наказания живут в {@link ModerationClient}: у них своя аудитория и свои коды
 * отказа, которые надо доносить до модератора дословно.
 */
public final class MasterClient {

    private final MasterHttp http;

    public MasterClient(AgentConfig config) {
        this(new MasterHttp(config));
    }

    public MasterClient(MasterHttp http) {
        this.http = http;
    }

    public MasterHttp http() {
        return http;
    }

    /**
     * Профиль игрока: роли, группы, наказания и доступ — одним запросом.
     *
     * @return пустой {@link Optional}, если игрока нет в базе мастера (404)
     */
    public Optional<PlayerProfile> player(UUID mcUuid) throws IOException, InterruptedException {
        return http.get("/api/agent/players/" + mcUuid, PlayerProfile.class);
    }

    /**
     * Профиль по нику. Нужен командам модерации: наказывают и того, кого сейчас
     * нет на сервере, а ванильный кэш имён знает только заходивших сюда.
     */
    public Optional<PlayerProfile> playerByName(String username) throws IOException, InterruptedException {
        return http.get("/api/agent/players/by-name/" + Uris.segment(username), PlayerProfile.class);
    }

    /**
     * Профили пачкой — для кадра «перечитать всех».
     *
     * <p>Неизвестные мастеру UUID в ответ не попадают: спрашиваем по списку
     * онлайна, где может оказаться и тот, кого мастер не знает.
     */
    public List<PlayerProfile> playersBatch(List<UUID> uuids) throws IOException, InterruptedException {
        PlayerProfile[] found =
                http.post("/api/agent/players/batch", new Batch(uuids), PlayerProfile[].class);
        return found == null ? List.of() : List.of(found);
    }

    /**
     * Сигнал жизни. Мастер считает сервер живым 90 секунд после последнего.
     *
     * <p>Вместе с числами едет полный состав. Он, а не события входа и выхода,
     * задаёт истину: канал рвётся, кадры теряются, и без сверки у мастера
     * копились бы вечно живые игроки.
     */
    public void heartbeat(ServerStatus status) throws IOException, InterruptedException {
        http.post(
                "/api/agent/heartbeat",
                new Heartbeat(
                        status.online(),
                        status.maxPlayers(),
                        status.version(),
                        List.copyOf(status.players()),
                        List.copyOf(status.vanished()),
                        status.tps(),
                        status.mspt(),
                        heapUsedMb(),
                        heapMaxMb(),
                        uptimeSeconds()));
    }

    /**
     * Каталог узлов прав, зарегистрированных на этом сервере.
     *
     * <p>Перечислить их заранее неоткуда: узлы приносят сами моды, и у каждой
     * сборки набор свой. Мастер по этому списку подсказывает в админке, а не
     * гадает по захардкоженному перечню.
     */
    public void reportNodes(Collection<String> nodes) throws IOException, InterruptedException {
        http.post("/api/agent/nodes", new Nodes(List.copyOf(nodes)));
    }

    public void createReport(UUID reporter, UUID target, String reason) throws IOException, InterruptedException {
        http.post("/api/agent/reports", new CreateReportPayload(reporter, target, reason));
    }

    private static long heapUsedMb() {
        Runtime runtime = Runtime.getRuntime();
        return (runtime.totalMemory() - runtime.freeMemory()) / MB;
    }

    private static long heapMaxMb() {
        return Runtime.getRuntime().maxMemory() / MB;
    }

    /**
     * Сколько живёт JVM. Не сколько живёт мир: агент запускается вместе с
     * процессом, и различить их изнутри нечем — да и оператору важно именно
     * «когда сервер последний раз поднимали».
     */
    private static long uptimeSeconds() {
        return java.lang.management.ManagementFactory.getRuntimeMXBean().getUptime() / 1000;
    }

    private static final long MB = 1024 * 1024;

    /**
     * Тело heartbeat. Имена полей сериализуются политикой Gson в snake_case.
     *
     * <p>Всё, кроме первых трёх, необязательно на стороне мастера: старый агент
     * обязан работать с новым мастером и наоборот.
     */
    private record Heartbeat(
            int online,
            int maxPlayers,
            String version,
            List<UUID> players,
            List<UUID> vanished,
            Double tps,
            Double mspt,
            long heapUsedMb,
            long heapMaxMb,
            long uptimeSecs) {}

    /** Тело отчёта об узлах прав. */
    private record Nodes(List<String> nodes) {}

    /** Тело батч-запроса профилей. */
    private record Batch(List<UUID> uuids) {}

    private record CreateReportPayload(UUID reporter, UUID target, String reason) {}
}
