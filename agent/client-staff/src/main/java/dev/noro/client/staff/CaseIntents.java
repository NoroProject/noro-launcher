package dev.noro.client.staff;

/**
 * Мод → лаунчер: намерения.
 *
 * <p>Мод говорит, чего хочет, и не знает, каким запросом это делается. URL
 * админки здесь нет и не будет — иначе правка ручки ломала бы jar, который уже
 * уехал к людям.
 *
 * <p>Действий в мире здесь тоже нет: телепорт, заморозку и слежку панель шлёт
 * теми же командами {@code /case …}, что и кнопки в чате.
 */
public sealed interface CaseIntents {

    /** Перечитать очередь с начала и без фильтра. */
    record RequestQueue() implements CaseIntents {}

    /**
     * Страница очереди с поиском.
     *
     * <p>Ищет мастер, а не панель: очередь на большом сервере в десяток видимых
     * строк не помещается, и фильтр по загруженной странице находил бы только
     * то, что уже на экране.
     *
     * @param query ник, сервер, модератор или номер дела; {@code null} — вся
     *     очередь
     * @param offset сколько дел пропустить от начала
     */
    record RequestQueuePage(String query, long offset) implements CaseIntents {}

    /** Лаунчер запомнит дело открытым и будет присылать его на каждое событие. */
    record OpenCase(String case_id) implements CaseIntents {}

    record CloseCase() implements CaseIntents {}

    record Claim(String case_id) implements CaseIntents {}

    record Release(String case_id) implements CaseIntents {}

    record Resolve(String case_id, String verdict, String resolution, String rule_code)
            implements CaseIntents {}

    record AddNote(String case_id, String text) implements CaseIntents {}

    record Punish(
            String case_id, String kind, String reason, String rule_code, Long duration_secs)
            implements CaseIntents {}

    /** Срез снимает агент — ответ приедет обновлённой карточкой, а не сюда. */
    record RequestChat(String case_id) implements CaseIntents {}

    record RequestInventory(String case_id) implements CaseIntents {}

    /** Кадр экрана в дело; PNG едет base64, канал текстовый. */
    record Attach(String case_id, String note, String png_base64) implements CaseIntents {}

    /**
     * Указание на сообщение в чате, а не само сообщение.
     *
     * <p>Клиент не источник доказательств: в дело поедет строка из буфера
     * агента, найденная по отправителю и времени. Подделать переписку правым
     * кликом поэтому невозможно.
     */
    record Quote(String case_id, String sender, String at, String hash) implements CaseIntents {}

    record Lookup(String username) implements CaseIntents {}
}
