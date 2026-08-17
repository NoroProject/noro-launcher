package dev.noro.agent.core;

import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;

/**
 * Кодирование того, что подставляется в путь.
 *
 * <p>Ник в команде набирает человек, и там встречается всё: пробел от опечатки,
 * слэш, знак вопроса. Без кодирования такой ник уезжает в другой маршрут и
 * возвращает невнятную ошибку вместо «нет такого игрока».
 */
final class Uris {

    private Uris() {}

    static String segment(String value) {
        // URLEncoder — форма для тела, а не для пути: пробел там становится
        // «+», который в сегменте пути так и останется плюсом.
        return URLEncoder.encode(value, StandardCharsets.UTF_8).replace("+", "%20");
    }
}
