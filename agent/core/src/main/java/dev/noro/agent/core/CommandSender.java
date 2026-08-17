package dev.noro.agent.core;

import java.util.UUID;

/**
 * Тот, кто набрал команду: игрок либо консоль.
 *
 * <p>Права спрашиваются здесь, а не у мастера: набор уже лежит у игрока с
 * логина, и лишний запрос ради «а можно ли» только задержал бы ответ. Решает
 * всё равно мастер — эта проверка нужна, чтобы отказ пришёл сразу и внятно.
 */
public interface CommandSender {

    /** MC UUID игрока; {@code null} — консоль сервера. */
    UUID uuid();

    String name();

    boolean has(String permission);

    void reply(String message);

    /** Консоль наказывает от имени сервера: человека за ней мастер не знает. */
    default boolean console() {
        return uuid() == null;
    }
}
