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
    /** Номер из реестра мастера; {@code 0} — мастер его не прислал. */
    private final int number;

    MasterRefusedException(int status, int number, String message) {
        super(message);
        this.status = status;
        this.number = number;
    }

    public int status() {
        return status;
    }

    public int number() {
        return number;
    }

    /**
     * Текст для модератора: причина и номер, который можно назвать.
     *
     * <p>Номер важнее, чем кажется: причина приходит на английском из мастера, а
     * номер одинаков в любом языке и его видно на скриншоте чата.
     */
    public String display() {
        return number > 0 ? getMessage() + " (" + number + ")" : getMessage();
    }
}
