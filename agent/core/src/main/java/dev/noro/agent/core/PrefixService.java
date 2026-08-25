package dev.noro.agent.core;

import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Плашки ролей: что показать конкретному игроку.
 *
 * <p>Плашка — картинка из ресурспака, и увидит её только тот, кто пак принял.
 * Отказавшемуся тот же символ показался бы белым квадратом, поэтому ему уходит
 * прежний текстовый префикс. Решение принимается на каждого зрителя отдельно —
 * иначе пришлось бы либо требовать пак от всех, либо мириться с квадратами.
 *
 * <p>Кто пак принял, известно точно: клиент отвечает на выдачу статусом. Гадать
 * по «мы же ему отправили» нельзя — пак ещё и не докачивается.
 */
public final class PrefixService {

    /** Шрифт плашек. Тот же, что кладётся в пак: имена обязаны совпадать. */
    public static final String FONT = "noro:prefix";

    private volatile PrefixPack pack = PrefixPack.none();
    private volatile Map<String, String> glyphs = Map.of();
    private final Set<UUID> accepted = ConcurrentHashMap.newKeySet();

    /** Новый состав пака от мастера. */
    public void pack(PrefixPack value) {
        this.pack = value == null ? PrefixPack.none() : value;
        this.glyphs = this.pack.byRole();
    }

    public PrefixPack pack() {
        return pack;
    }

    /** Ответ клиента на выдачу пака. */
    public void status(UUID player, boolean ok) {
        if (ok) {
            accepted.add(player);
        } else {
            accepted.remove(player);
        }
    }

    public void forget(UUID player) {
        accepted.remove(player);
    }

    public boolean has(UUID player) {
        return accepted.contains(player);
    }

    /**
     * Префикс игрока глазами зрителя.
     *
     * <p>Зритель не учитывается намеренно. Чтобы показать плашку одному и
     * звёздочку другому, текст сообщения надо собирать заново на каждого
     * получателя, а он собирается один раз до рассылки — и точки, где его можно
     * пересобрать тем же форматом, в чате не нашлось. Пак при этом едет игроку
     * вместе со сборкой лаунчера, так что без него остаётся тот, кто зашёл
     * чужим клиентом; он увидит пустой глиф вместо плашки.
     *
     * @param viewer кому показываем — сейчас не влияет ни на что
     * @return символ плашки либо {@code null}, если у роли её нет
     */
    public String glyph(UUID viewer, RoleInfo role) {
        return role == null ? null : glyphs.get(role.name());
    }

    /** Есть ли вообще что выдавать: пустой пак игрокам не шлётся. */
    public boolean usable() {
        return pack.usable();
    }
}
