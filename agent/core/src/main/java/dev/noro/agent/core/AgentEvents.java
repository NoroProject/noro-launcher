package dev.noro.agent.core;

import com.google.gson.annotations.SerializedName;

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

    /** Модератор взял дело командой в игре. Замок ставит мастер. */
    public void caseClaim(java.util.UUID caseId, UUID moderator) {
        send(new CaseClaim("case_claim", caseId, moderator));
    }

    /** Что модератор сделал в разборе: телепорт, слежка, заморозка. */
    public void caseAction(
            java.util.UUID caseId, UUID moderator, String kind, java.util.Map<String, String> payload) {
        send(new CaseAction("case_action", caseId, moderator, kind, payload));
    }

    /** Срез чата: окно вокруг события, а не весь буфер. */
    public void caseChatSlice(java.util.UUID caseId, java.util.List<ChatRing.Entry> messages) {
        java.util.List<Line> lines = new java.util.ArrayList<>();
        for (ChatRing.Entry entry : messages) {
            lines.add(new Line(entry.at(), entry.sender(), entry.senderName(), entry.channel(), entry.content()));
        }
        send(new CaseChatSlice("case_chat_slice", caseId, lines));
    }

    public void caseInventory(
            java.util.UUID caseId, UUID moderator, java.util.List<GameBridge.Slot> items) {
        send(new CaseInventory("case_inventory", caseId, moderator, java.util.Map.of("items", items)));
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

    private record CaseClaim(String type, @SerializedName("case") java.util.UUID caseId, UUID moderator) {}

    private record CaseAction(
            String type,
            @SerializedName("case") java.util.UUID caseId,
            UUID moderator,
            String kind,
            java.util.Map<String, String> payload) {}

    private record CaseChatSlice(String type, @SerializedName("case") java.util.UUID caseId, java.util.List<Line> messages) {}

    private record Line(
            java.time.Instant at, UUID sender, String senderName, String channel, String content) {}

    private record CaseInventory(
            String type,
            @SerializedName("case") java.util.UUID caseId,
            UUID moderator,
            java.util.Map<String, Object> items) {}
}
