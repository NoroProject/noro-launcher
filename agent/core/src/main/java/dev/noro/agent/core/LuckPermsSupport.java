package dev.noro.agent.core;

import org.slf4j.Logger;

/**
 * Единственная дверь к LuckPerms. За ней он может отсутствовать.
 *
 * <p>Порядок здесь важен и неочевиден. Сначала {@link #available()} проверяет
 * наличие класса, ничего его не инициализируя. Только потом трогается
 * {@link Holder}, и лишь в этот момент JVM грузит классы, где типы LuckPerms
 * упоминаются. Разнести это по разным классам обязательно: константы резолвятся
 * лениво, а вот верификация метода может потребовать загрузки типов из его тела
 * целиком — и тогда падение случится раньше любого `catch`.
 */
public final class LuckPermsSupport {

    private LuckPermsSupport() {}

    /**
     * @return {@code null}, если LuckPerms не установлен — это штатный режим:
     *         контроль доступа и heartbeat работают и без него
     */
    public static RoleApplier tryCreate(Logger log) {
        if (!available()) {
            log.warn("LuckPerms not found — access control stays on, role sync is off");
            return null;
        }
        try {
            return Holder.create(log);
        } catch (Throwable e) {
            log.warn("LuckPerms present but unusable, role sync is off: {}", e.toString());
            return null;
        }
    }

    /** `initialize = false`: класс не инициализируем, только проверяем наличие. */
    private static boolean available() {
        try {
            Class.forName(
                    "net.luckperms.api.LuckPermsProvider",
                    false,
                    LuckPermsSupport.class.getClassLoader());
            return true;
        } catch (Throwable e) {
            return false;
        }
    }

    /** Грузится только после того, как {@link #available()} подтвердил LuckPerms. */
    private static final class Holder {
        static RoleApplier create(Logger log) {
            return new RoleSync(net.luckperms.api.LuckPermsProvider.get(), log);
        }
    }
}
