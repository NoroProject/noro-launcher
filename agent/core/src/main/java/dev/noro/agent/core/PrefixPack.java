package dev.noro.agent.core;

import com.google.gson.annotations.SerializedName;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

/**
 * Пак плашек, каким его отдаёт мастер: адрес, контрольная сумма и таблица
 * символов.
 *
 * <p>Плашка роли — картинка в ресурспаке, а в игре она один символ в шрифте
 * {@code noro:prefix}. Кто пак принял, тот видит картинку; остальным агент
 * подставляет обычную иконку роли, поэтому таблица нужна и тем, и другим.
 *
 * @param url откуда игроку скачать пак
 * @param sha1 чем проверить закачку — его же требует ванильная выдача пака
 * @param glyphs роль → символ
 */
public record PrefixPack(String url, String sha1, List<Glyph> glyphs) {

    /** Пары «роль — символ». Отдельной записью, потому что так их шлёт мастер. */
    public record Glyph(String role, String symbol) {}

    /** Пустой пак: мастер не ответил или ни одной роли с плашкой нет. */
    public static PrefixPack none() {
        return new PrefixPack("", "", List.of());
    }

    public boolean usable() {
        return url != null && !url.isBlank() && sha1 != null && !sha1.isBlank();
    }

    /** Таблица для быстрого поиска: имя роли — символ плашки. */
    public Map<String, String> byRole() {
        return glyphs == null
                ? Map.of()
                : glyphs.stream()
                        .filter(g -> g.role() != null && g.symbol() != null)
                        .collect(Collectors.toMap(Glyph::role, Glyph::symbol, (a, b) -> a));
    }
}
