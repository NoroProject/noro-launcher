package dev.noro.agent.core;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Профили игроков, которые сейчас на сервере.
 *
 * <p>За профилем агент ходит один раз — на логине, и там же его тратит на роли
 * и права. Плейсхолдеры спрашивают те же данные когда угодно и сколько угодно
 * раз, поэтому ответ мастера надо пережить дольше входа. Второй запрос вместо
 * кэша не годится: плейсхолдер дёргают на каждый тик отрисовки таба.
 *
 * <p>Карта конкурентная: пишет поток логина, читает игровой.
 */
public final class ProfileCache {

    private final Map<UUID, PlayerProfile> byPlayer = new ConcurrentHashMap<>();

    public void remember(UUID uuid, PlayerProfile profile) {
        if (profile != null) {
            byPlayer.put(uuid, profile);
        }
    }

    /** @return {@code null}, если игрока нет на сервере или мастер был недоступен */
    public PlayerProfile get(UUID uuid) {
        return uuid == null ? null : byPlayer.get(uuid);
    }

    /**
     * Оборванный между логином и входом в мир сеанс — единственный случай, когда
     * запись остаётся: события выхода по нему не будет. Следующая попытка того же
     * игрока её перезапишет, поэтому карта растёт не быстрее списка знакомых UUID.
     */
    public void forget(UUID uuid) {
        byPlayer.remove(uuid);
    }
}
