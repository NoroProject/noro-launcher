package dev.noro.agent.core;

import com.google.gson.Gson;
import java.util.UUID;

/**
 * События игры, которые агент сообщает мастеру.
 *
 * <p>Канал сюда — только ускорение. Мастер сверяет состав по heartbeat, где
 * едет полный список, а эти кадры нужны, чтобы список сошёлся сразу, а не через
 * полминуты. Поэтому ни один вызов ничего не возвращает и ничего не ждёт:
 * потеря кадра допустима по построению.
 *
 * <p>Зовут из игрового потока — вход и выход случаются именно там. Работа с
 * сетью полностью уходит в {@link LinkOutbox}.
 */
public final class AgentEvents {

    private final LinkOutbox outbox;
    private final Gson gson;

    AgentEvents(LinkOutbox outbox, Gson gson) {
        this.outbox = outbox;
        this.gson = gson;
    }

    /**
     * @param ipHash уже хеш: сырой адрес мастеру не нужен, а хранить его —
     *               лишний повод объясняться
     * @param vanished скрыт ванишем — в публичном онлайне его быть не должно
     */
    public void playerJoin(UUID uuid, String ipHash, boolean vanished) {
        send(new PlayerJoin("player_join", uuid, ipHash, vanished));
    }

    public void playerLeave(UUID uuid, String reason) {
        send(new PlayerLeave("player_leave", uuid, reason));
    }

    /** Игровой поток не двигался столько секунд — сервер завис. */
    public void tickStall(int stalledSeconds) {
        send(new TickStall("tick_stall", stalledSeconds));
    }

    private void send(Object frame) {
        outbox.offer(gson.toJson(frame));
    }

    // Имена полей уезжают в snake_case политикой Gson — той же, что у HTTP.
    // Поля должны совпадать с `FromAgent` в
    // `crates/master/src/agent_link/proto.rs`.

    private record PlayerJoin(String type, UUID uuid, String ipHash, boolean vanished) {}

    private record PlayerLeave(String type, UUID uuid, String reason) {}

    private record TickStall(String type, int stalledSecs) {}
}
