package dev.noro.agent.core;

import java.util.Collection;
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
    private static final PermissionNodeCatalog NODES = new PermissionNodeCatalog();

    /**
     * Кто отвечает на вопрос «замучен ли игрок». Ставит агент при старте.
     *
     * <p>Отдельной точкой, а не плейсхолдером: чат-моды перехватывают сообщение
     * сами (на NeoForge иначе и нельзя — StyledChat уводит чат из ванильного
     * пути), и им нужен готовый текст отказа, а не поле профиля.
     */
    private static volatile Mutes mutes;

    private NoroAgentApi() {}

    /** Источник текста отказа при муте. Реализует агент. */
    public interface Mutes {
        /**
         * @param actionbar нужен короткий текст для строки над хотбаром
         * @return текст отказа либо {@code null}, если игрок не замучен
         */
        String notice(UUID uuid, boolean actionbar);
    }

    /** Агент подключает свой источник мутов. Не для чужого кода. */
    public static void attachMutes(Mutes source) {
        mutes = source;
    }

    /**
     * Замучен ли игрок и что ему показать.
     *
     * <p>Чат-мод спрашивает это перед отправкой сообщения: {@code null} —
     * говорить можно. Текст уже собран по шаблону мастера, с цветами и
     * подстановками; разбирать его чат-моду не нужно, только показать.
     *
     * @param actionbar {@code true} — короткая версия для строки над хотбаром
     */
    public static String muteNotice(UUID uuid, boolean actionbar) {
        Mutes source = mutes;
        return source == null ? null : source.notice(uuid, actionbar);
    }

    /** Хранилище, которое агент наполняет на входе игрока. Не для чужого кода. */
    public static ProfileCache cache() {
        return CACHE;
    }

    /**
     * Каталог узлов прав, по которому мастер подсказывает в админке.
     *
     * <p>Реестра узлов нет на Fabric ни у одного мода, а на Forge и NeoForge он
     * заполняется раньше, чем большинство модов успевает прочитать свой конфиг.
     * Поэтому узлы принимаются здесь, а не только собираются с платформы.
     */
    public static PermissionNodeCatalog permissionNodes() {
        return NODES;
    }

    /**
     * Заявляет узлы прав, которые понимает чужой мод или плагин.
     *
     * <p>Звать можно когда угодно и сколько угодно раз: каталог сам решит, надо
     * ли отправлять. Принимает и отдаёт только типы JDK, поэтому годится для
     * вызова через reflection наравне с {@link #value(UUID, String)}.
     *
     * @param nodes полные имена узлов, например {@code nbitchat.chat.local}
     */
    public static void registerPermissionNodes(Collection<String> nodes) {
        NODES.register(nodes);
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
    /**
     * Плашки ролей: их состав и то, кто согласился их видеть.
     *
     * <p>Ставится агентом при старте. Пока не поставлена, префикс остаётся
     * текстовым — ровно как до появления плашек.
     */
    private static volatile PrefixService prefixes;

    public static void attachPrefixes(PrefixService service) {
        prefixes = service;
    }

    /**
     * Кому сейчас собирается текст.
     *
     * <p>Плейсхолдеру {@code %noro:prefix%} зритель нужен, а передать его через
     * API плейсхолдеров некуда: там на входе только автор. Поток здесь тот же —
     * чат рассылается получателям по очереди в серверном потоке, — поэтому
     * значение живёт ровно на время сборки одного сообщения.
     *
     * <p>Ставит и снимает его тот, кто рассылает: форки TAB и StyledChat.
     */
    private static final ThreadLocal<UUID> VIEWER = new ThreadLocal<>();

    public static void viewer(UUID uuid) {
        if (uuid == null) {
            VIEWER.remove();
        } else {
            VIEWER.set(uuid);
        }
    }

    public static UUID viewer() {
        return VIEWER.get();
    }

    /**
     * Префикс игрока глазами зрителя.
     *
     * <p>Двумя игроками, а не одним, потому что плашка — картинка из
     * ресурспака: у кого пака нет, тот увидел бы на её месте белый квадрат.
     * Поэтому принявшему уходит символ в шрифте {@code noro:prefix}, а всем
     * остальным — прежний текстовый префикс.
     *
     * <p>Зовут это форки TAB и StyledChat: чат и таб они собирают на каждого
     * получателя отдельно, и подставить разное там есть куда.
     *
     * @param viewer кому показываем; {@code null} — не знаем, значит без плашки
     * @return строка в разметке MiniMessage, никогда не {@code null}
     */
    public static String prefixFor(UUID uuid, UUID viewer) {
        return badgeOr(prefixRole(uuid), viewer);
    }

    /**
     * Плашка роли либо её текстовый префикс.
     *
     * <p>Роль передаётся готовой, а не ищется по игроку: там, где она уже
     * разобрана, второй поход в кэш давал бы пусто для профиля, которого в кэше
     * нет, — ровно так и ломался плейсхолдер.
     */
    public static String badgeOr(RoleInfo role, UUID viewer) {
        if (role == null) {
            return "";
        }
        PrefixService service = prefixes;
        String glyph = service == null ? null : service.glyph(viewer, role);
        // Пробел после плашки: без него картинка упирается в ник и читается
        // как одно слово. В самой картинке его не заложить — она обрезана по
        // краю, и лишний столбец пикселей сдвинул бы фон, а не текст.
        return glyph == null
                ? role.prefixText()
                : "<font:" + PrefixService.FONT + ">" + glyph + "</font> ";
    }

    public static String value(UUID uuid, String key) {
        return PlaceholderValues.resolve(profile(uuid), key);
    }
}
