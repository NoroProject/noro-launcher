package dev.noro.client.player;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * Что игрокский мод знает прямо сейчас.
 *
 * <p>Как и у панели разбора: экраны только читают состояние на отрисовке, а
 * кладёт его сюда канал. Свод правил и наказания живут до выхода из игры —
 * перечитывать их на каждое открытие экрана незачем, они меняются редко.
 */
public final class PlayerState {

    private volatile List<PlayerFrames.Category> categories = List.of();
    private volatile List<PlayerFrames.Rule> rules = List.of();
    private volatile List<PlayerFrames.Sanction> sanctions = List.of();
    private volatile List<PlayerFrames.Punishment> punishments = List.of();

    /** Кадр от лаунчера. Забытый вид не скомпилируется — на то и sealed. */
    public void accept(PlayerFrames frame) {
        switch (frame) {
            case PlayerFrames.Rules r -> {
                categories = or(r.categories());
                rules = or(r.rules());
                sanctions = or(r.sanctions());
            }
            case PlayerFrames.OwnPunishments p -> punishments = or(p.punishments());
        }
    }

    private static <T> List<T> or(List<T> value) {
        return value == null ? List.of() : value;
    }

    public boolean rulesLoaded() {
        return !rules.isEmpty();
    }

    /** Пункты раздела по порядку. `null` — те, у кого раздела нет. */
    public List<PlayerFrames.Rule> rulesOf(String categoryId) {
        List<PlayerFrames.Rule> out = new ArrayList<>();
        for (PlayerFrames.Rule rule : rules) {
            boolean mine = categoryId == null
                    ? rule.category_id() == null
                    : categoryId.equals(rule.category_id());
            if (mine) {
                out.add(rule);
            }
        }
        out.sort(Comparator.comparingInt(PlayerFrames.Rule::sort_order));
        return out;
    }

    /** Разделы по порядку; в конце — псевдораздел для пунктов без раздела. */
    public List<PlayerFrames.Category> categories() {
        List<PlayerFrames.Category> out = new ArrayList<>(categories);
        out.sort(Comparator.comparingInt(PlayerFrames.Category::sort_order));
        return out;
    }

    /** Вилки наказаний по пункту: правило без последствий читается неполно. */
    public List<PlayerFrames.Sanction> sanctionsOf(String ruleId) {
        List<PlayerFrames.Sanction> out = new ArrayList<>();
        for (PlayerFrames.Sanction s : sanctions) {
            if (s.rule_id().equals(ruleId)) {
                out.add(s);
            }
        }
        return out;
    }

    /** Свои наказания: сначала действующие, потом история — она тоже нужна. */
    public List<PlayerFrames.Punishment> punishments() {
        List<PlayerFrames.Punishment> out = new ArrayList<>(punishments);
        out.sort(Comparator
                .comparing(PlayerFrames.Punishment::active)
                .reversed()
                .thenComparing(PlayerFrames.Punishment::created_at, Comparator.reverseOrder()));
        return out;
    }

    public long activeCount() {
        return punishments.stream().filter(PlayerFrames.Punishment::active).count();
    }
}
