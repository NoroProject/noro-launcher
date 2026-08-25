package dev.noro.client.staff;

import com.google.gson.JsonObject;
import dev.noro.client.link.Bridge;
import dev.noro.client.link.Feature;
import dev.noro.client.link.Frames;
import java.util.Map;

/**
 * Разбор дел как одна из функций мода.
 *
 * <p>Здесь и только здесь живёт словарь кадров про дела: транспорт про них не
 * знает, а следующая функция объявит свои и получит их тем же способом.
 */
public final class Cases implements Feature {

    private static final Map<String, Class<? extends CaseFrames>> INBOUND = Map.of(
            "Ready", CaseFrames.Ready.class,
            "Queue", CaseFrames.Queue.class,
            "Case", CaseFrames.Case.class,
            "Dossier", CaseFrames.Dossier.class,
            "Inventory", CaseFrames.Inventory.class,
            "Rejected", CaseFrames.Rejected.class,
            "Notice", CaseFrames.Notice.class);

    private final CaseState state = new CaseState();
    private Bridge bridge;

    @Override
    public String id() {
        return "cases";
    }

    public CaseState state() {
        return state;
    }

    /** Намерение наружу. Канала нет — намерение просто некуда деть. */
    public void send(CaseIntents intent) {
        if (bridge != null) {
            bridge.send(intent);
        }
    }

    @Override
    public boolean accept(String type, JsonObject envelope) {
        Class<? extends CaseFrames> target = INBOUND.get(type);
        if (target == null) {
            return false;
        }
        state.accept(Frames.data(envelope, target));
        return true;
    }

    @Override
    public void connected(Bridge bridge) {
        this.bridge = bridge;
        // Очередь нужна сразу: панель открывают, чтобы взять следующее дело.
        bridge.send(new CaseIntents.RequestQueue());
    }

    @Override
    public void disconnected() {
        bridge = null;
        state.disconnected();
    }
}
