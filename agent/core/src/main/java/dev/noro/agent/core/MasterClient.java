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

    /** Сигнал жизни. Мастер считает сервер живым 90 секунд после последнего. */
    public void heartbeat(int online, int maxPlayers, String version) throws IOException, InterruptedException {
        http.post("/api/agent/heartbeat", new Heartbeat(online, maxPlayers, version));
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

    /** Тело heartbeat. Имена полей сериализуются политикой Gson в snake_case. */
    private record Heartbeat(int online, int maxPlayers, String version) {}

    /** Тело отчёта об узлах прав. */
    private record Nodes(List<String> nodes) {}
}
