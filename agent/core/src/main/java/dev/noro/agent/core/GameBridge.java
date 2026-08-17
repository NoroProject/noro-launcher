package dev.noro.agent.core;

import java.util.Collection;
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
}
