package dev.noro.agent.core;

import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

/**
 * Всё, что модерации нужно от игры. Три реализации: Paper, Fabric, NeoForge.
 *
 * <p>Методы зовут откуда угодно — из потока WebSocket, из фонового запроса к
 * мастеру, из команды. Уйти в игровой поток обязана реализация: кто именно
 * считается «главным», знает только платформа.
 */
public interface GameBridge {

    /** @return UUID игрока, который сейчас на сервере */
    Optional<UUID> onlineUuid(String name);

    /** Ники тех, кто в сети, — для автодополнения команд. */
    Collection<String> onlineNames();

    /** Отключить с экраном. Игрока может уже не быть — это не ошибка. */
    void kick(UUID uuid, String message);

    /** Личное сообщение игроку. */
    void tell(UUID uuid, String message);

    /** Сообщение в actionbar над хотбаром игрока. */
    void actionbar(UUID uuid, String message);

    /** Сообщение всем на сервере. */
    void announce(String message);

    /** Сообщение только игрокам с указанным правом (пермишеном). */
    default void announceToPermission(String permission, String message) {
        announce(message);
    }

    /**
     * Инструменты разбора жалобы. У них общая судьба: платформа, где такого
     * API нет, отвечает {@code false}, и кнопка в меню просто скажет об этом.
     * Ломать весь режим из-за одной невозможной операции незачем.
     */
    default boolean teleport(UUID who, String world, double x, double y, double z) {
        return false;
    }

    /** Телепорт к игроку — он мог выйти, и это не ошибка. */
    default boolean teleportTo(UUID who, UUID target) {
        return false;
    }

    /** Где игрок сейчас: мир и координаты, чтобы вернуть его назад. */
    default Optional<Position> position(UUID who) {
        return Optional.empty();
    }

    /**
     * Открыть модератору инвентарь цели контейнером — как {@code /invsee}.
     *
     * <p>Не снимок: предметы должны быть настоящими, иначе изъять улику или
     * вернуть украденное можно только руками через команды. Правку видит и сам
     * игрок — окно смотрит в его инвентарь, а не в копию.
     *
     * @return {@code false} — платформа так не умеет, и кнопка честно скажет
     */
    default boolean openInventory(UUID viewer, UUID target) {
        return false;
    }

    /** То же для эндер-сундука: половина спрятанного лежит там. */
    default boolean openEnderChest(UUID viewer, UUID target) {
        return false;
    }

    /**
     * Выдать игроку ресурспак с плашками ролей.
     *
     * <p>Не в папку, а выдачей сервера: пак в папке подхватывается только при
     * запуске клиента, а роли правят на ходу. С 1.20.3 клиент применяет
     * выданный пак не перезапускаясь.
     *
     * @return {@code false} — платформа так не умеет, плашек у игрока не будет
     */
    default boolean sendResourcePack(UUID who, String url, String sha1) {
        return false;
    }

    /** Свечение цели — видимое только одному зрителю. */
    default boolean glow(UUID viewer, UUID target, boolean on) {
        return false;
    }

    /** Наблюдение: камера зрителя следует за целью. */
    default boolean spectate(UUID viewer, UUID target) {
        return false;
    }

    /** Вернуть зрителя из наблюдения в обычный режим. */
    default boolean stopSpectate(UUID viewer) {
        return false;
    }

    /**
     * Содержимое инвентаря слотами.
     *
     * <p>Слотами, а не строками «предмет xN»: из локализованного имени предмет
     * не восстановить, а панель разбора рисует настоящие иконки. Имя всё равно
     * едет рядом — тем, кто показывает снимок текстом, оно нужно готовым.
     */
    default List<Slot> inventory(UUID who) {
        return List.of();
    }

    /**
     * Занятый слот инвентаря.
     *
     * @param slot номер слота: по нему видно, что в руке, а что в рюкзаке
     * @param id идентификатор предмета вида {@code minecraft:diamond_pickaxe}
     * @param count сколько штук
     * @param name готовое имя для тех, кто показывает снимок текстом
     * @param nbt предмет целиком в JSON — с зачарованиями, прочностью и
     *     переименованием. По нему панель рисует настоящую иконку со всеми
     *     подписями; {@code null}, если платформа так не умеет, и тогда
     *     остаётся текст
     */
    record Slot(int slot, String id, int count, String name, String nbt) {

        /** Строкой — для чата и ленты: «Алмазная кирка x1». */
        public String label() {
            return name + " x" + count;
        }
    }

    /** Точка в мире: куда телепортировали и куда возвращаться. */
    record Position(String world, double x, double y, double z) {}
}
