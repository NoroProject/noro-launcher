package dev.noro.agent.core;

import java.time.Duration;
import java.time.Instant;
import java.util.Map;
import java.util.Set;
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
 *
 * <p>Кэш обязан уметь ошибаться и исправляться. Кадр о снятии мог не дойти —
 * канал переподключался, сервер моргнул, — и тогда игрок молчит навсегда, а
 * причины этому в игре не видно. Поэтому запись помнит, когда её проверяли:
 * {@link #stale(UUID, Duration)} показывает, что пора переспросить мастера.
 */
public final class MuteRegistry {

    private final Map<UUID, PunishmentInfo> muted = new ConcurrentHashMap<>();
    private final Map<UUID, Instant> checked = new ConcurrentHashMap<>();

    /** {@code null} в наказании снимает мут — так приходит снятие с мастера. */
    public void remember(UUID uuid, PunishmentInfo mute) {
        if (uuid == null) {
            return;
        }
        checked.put(uuid, Instant.now());
        if (mute == null || !mute.active()) {
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

    /**
     * Пора ли сверить мут игрока с мастером.
     *
     * <p>Спрашиваем только про замученных: у остальных сверять нечего, а лишний
     * запрос на каждого игрока превратил бы полный сервер в поток обращений.
     */
    public boolean stale(UUID uuid, Duration every) {
        if (uuid == null || !muted.containsKey(uuid)) {
            return false;
        }
        Instant last = checked.get(uuid);
        return last == null || last.plus(every).isBefore(Instant.now());
    }

    /** Кого стоит проверить: копия ключей, чтобы обход не держал карту. */
    public Set<UUID> mutedPlayers() {
        return Set.copyOf(muted.keySet());
    }

    public void forget(UUID uuid) {
        muted.remove(uuid);
        checked.remove(uuid);
    }
}
