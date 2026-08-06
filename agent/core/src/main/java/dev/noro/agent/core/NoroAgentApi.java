package dev.noro.agent.core;

import java.util.UUID;

/**
 * Профиль игрока для чужих модов и плагинов — чат-форматтеров, табов, меню.
 *
 * <p>Существует потому, что плейсхолдеров хватает не всюду: под Forge и NeoForge
 * не существует ни PlaceholderAPI, ни Text Placeholder API, и там это
 * единственный способ узнать роль игрока, не разбирая {@code §x}-строку из
 * LuckPerms обратно в hex.
 *
 * <p>Данные приходят готовыми: {@link RoleInfo#color()} — это {@code #rrggbb},
 * иконка лежит отдельно от цвета, а {@link RoleInfo#sortOrder()} совпадает с
 * весом префикса в LuckPerms и с порядком на сайте.
 *
 * <p>Кэш живёт здесь, а не в платформенной части: агент один на JVM, а
 * отдельная фаза установки означала бы окно, в котором API уже виден, но ещё
 * пуст, — и звонящий не смог бы отличить это от «игрок офлайн».
 */
public final class NoroAgentApi {

    private static final ProfileCache CACHE = new ProfileCache();

    private NoroAgentApi() {}

    /** Хранилище, которое агент наполняет на входе игрока. Не для чужого кода. */
    public static ProfileCache cache() {
        return CACHE;
    }

    /**
     * @return {@code null}, если игрока нет на сервере или мастер был недоступен
     *         в открытом режиме — тогда роли неизвестны, и это не то же самое,
     *         что «роли отсутствуют»
     */
    public static PlayerProfile profile(UUID uuid) {
        return CACHE.get(uuid);
    }

    /**
     * Роль, чей префикс виден в игре: старшая **из тех, у кого есть иконка**.
     * У старшей роли иконки может не быть — тогда показывается следующая, и это
     * то же правило, по которому LuckPerms выбирает префикс по весам.
     *
     * @return {@code null}, если у игрока нет ни одной роли с иконкой
     */
    public static RoleInfo prefixRole(UUID uuid) {
        return PlaceholderValues.top(profile(uuid), true);
    }

    /** Старшая роль по {@code sort_order}, даже если она без иконки. */
    public static RoleInfo topRole(UUID uuid) {
        return PlaceholderValues.top(profile(uuid), false);
    }

    /**
     * Готовое значение по ключу плейсхолдера — те же имена, что у
     * {@code %noro_…%}: {@code role_color} даёт {@code #ff8c82},
     * {@code prefix} — собранную legacy-строку, {@code has_role_admin} —
     * {@code true}/{@code false}.
     *
     * <p>Годится для вызова через reflection, когда мод не хочет зависеть от
     * классов агента: и ключ, и ответ здесь — обычные строки.
     *
     * @return {@code null} на неизвестный ключ или неизвестного игрока
     */
    public static String value(UUID uuid, String key) {
        return PlaceholderValues.resolve(profile(uuid), key);
    }
}
