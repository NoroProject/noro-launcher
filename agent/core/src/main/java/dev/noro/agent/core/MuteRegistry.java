package dev.noro.agent.core;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Кто сейчас замучен на этом сервере.
 *
 * <p>Спрашивают на каждое сообщение в чат, поэтому ходить за ответом к мастеру
 * нельзя: чат встанет на время сетевого запроса. Запись появляется на входе
 * игрока (из профиля) и кадром живого канала, если мут выдали, пока игрок в
 * игре.
 *
 * <p>Истёкший мут снимается здесь же, при проверке: срок кончается сам по себе,
 * и события об этом от мастера не будет.
 */
public final class MuteRegistry {

    private final Map<UUID, PunishmentInfo> muted = new ConcurrentHashMap<>();

    /** {@code null} в наказании снимает мут — так приходит снятие с мастера. */
    public void remember(UUID uuid, PunishmentInfo mute) {
        if (uuid == null) {
            return;
        }
        if (mute == null) {
            muted.remove(uuid);
        } else {
            muted.put(uuid, mute);
        }
    }

    /** @return действующий мут либо {@code null} */
    public PunishmentInfo active(UUID uuid) {
        if (uuid == null) {
            return null;
        }
        PunishmentInfo mute = muted.get(uuid);
        if (mute == null) {
            return null;
        }
        if (!mute.active()) {
            muted.remove(uuid, mute);
            return null;
        }
        return mute;
    }

    public void forget(UUID uuid) {
        muted.remove(uuid);
    }
}
