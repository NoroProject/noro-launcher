package dev.noro.agent.core;

import java.io.IOException;

/**
 * Мастер отказал по существу: «правило допускает только до 7d», «нет такого
 * игрока».
 *
 * <p>Отдельно от сетевых сбоев, потому что это ответ модератору, а не поломка:
 * текст надо показать в чате дословно, а не прятать за «master is unreachable».
 */
public final class MasterRefusedException extends IOException {

    private final int status;

    MasterRefusedException(int status, String message) {
        super(message);
        this.status = status;
    }

    public int status() {
        return status;
    }
}
