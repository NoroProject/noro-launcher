package dev.noro.client.staff;

import java.util.List;
import java.util.Map;

/**
 * Лаунчер → мод: состояние.
 *
 * <p>Мод рисует то, что прислали, и ничего не выводит сам. {@code sealed} здесь
 * не украшение: забытый кадр в {@code switch} перестаёт компилироваться — ровно
 * как незакрытый {@code match} на той стороне.
 */
public sealed interface CaseFrames {

    /**
     * Рукопожатие принято. Права — чтобы не рисовать кнопки, которых всё равно
     * не дадут нажать. Это удобство, а не защита: решает мастер.
     */
    record Ready(int protocol, String username, String locale, List<String> permissions)
            implements CaseFrames {

        /** Права приходят с wildcard'ами: у админа это один {@code *}. */
        public boolean has(String required) {
            if (permissions == null) {
                return false;
            }
            for (String p : permissions) {
                if (p.equals("*") || p.equals(required)) {
                    return true;
                }
                if (p.endsWith(".*") && required.startsWith(p.substring(0, p.length() - 1))) {
                    return true;
                }
            }
            return false;
        }
    }

    /**
     * Страница очереди.
     *
     * <p>{@code total} — сколько дел подходит под фильтр целиком, а не сколько
     * приехало: по нему панель рисует «11–20 из 348» и знает, есть ли ещё.
     * {@code query} и {@code offset} — эхо запроса, по которому страница
     * собрана: без него кадр в ответ на перелистывание не отличить от
     * обновления очереди.
     */
    record Queue(List<CaseModels.Brief> cases, long total, long offset, String query)
            implements CaseFrames {}

    /** Карточка целиком, включая срез чата: он часть карточки, как и на сайте. */
    record Case(CaseModels.View view) implements CaseFrames {}

    record Dossier(CaseModels.Dossier dossier) implements CaseFrames {}

    /**
     * Снимок инвентаря отдельным кадром, хотя он же лежит событием в ленте: в
     * ленте это сырой JSON, и разбирать его здесь — работа на ровном месте.
     */
    record Inventory(String case_id, List<CaseModels.Slot> items) implements CaseFrames {}

    /**
     * Мастер отказал. {@code intent} — имя кадра, на который отказ; {@code
     * reason} — ключ перевода, а не готовый текст.
     *
     * <p>{@code number} — номер из реестра мастера, {@code 0} если отказ не от
     * него. Причина переведена в моде и на каждом языке своя, а номер одинаков
     * везде и читается со скриншота — по нему и находят место в коде.
     */
    record Rejected(String intent, String reason, int number) implements CaseFrames {}

    /** Сказать словами — ключом, как уведомления мастера. */
    record Notice(String key, Map<String, String> args) implements CaseFrames {}
}
