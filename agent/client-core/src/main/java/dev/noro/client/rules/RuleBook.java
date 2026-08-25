package dev.noro.client.rules;

import com.google.gson.JsonObject;
import dev.noro.client.link.Bridge;
import dev.noro.client.link.Feature;
import dev.noro.client.link.Frames;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;

/**
 * Свод правил — общий для всех модов Noro.
 *
 * <p>Живёт в ядре, а не в панели разбора: правила нужны и модератору при выдаче
 * наказания, и игроку, чтобы прочитать, за что его наказали. Два кэша одного и
 * того же расходились бы ровно в тот момент, когда свод правят.
 *
 * <p>Свод публичен намеренно — на него ссылается каждый бан, и забаненный
 * обязан прочитать, за что именно. Прав на него не нужно.
 */
public final class RuleBook implements Feature {

    /** Пункт свода. */
    public record Rule(String id, String category_id, String code, String title,
                       String description, int sort_order) {}

    /** Вилка наказания по пункту: что за него бывает и в каких границах. */
    public record Sanction(String rule_id, String kind, Long min_minutes, Long max_minutes) {}

    private record Payload(List<Rule> rules, List<Sanction> sanctions) {}

    private volatile List<Rule> rules = List.of();
    private volatile List<Sanction> sanctions = List.of();
    private Bridge bridge;

    @Override
    public String id() {
        return "rules";
    }

    @Override
    public boolean accept(String type, JsonObject envelope) {
        if (!"Rules".equals(type)) {
            return false;
        }
        Payload payload = Frames.data(envelope, Payload.class);
        rules = payload.rules() == null ? List.of() : payload.rules();
        sanctions = payload.sanctions() == null ? List.of() : payload.sanctions();
        return true;
    }

    @Override
    public void connected(Bridge bridge) {
        this.bridge = bridge;
        request();
    }

    @Override
    public void disconnected() {
        bridge = null;
    }

    /** Перечитать свод. Меняется он редко, поэтому спрашиваем по надобности. */
    public void request() {
        if (bridge != null) {
            bridge.send(new RequestRules());
        }
    }

    /** Намерение живёт рядом со сводом: больше его никто не шлёт. */
    public record RequestRules() {}

    public boolean loaded() {
        return !rules.isEmpty();
    }

    /** Пункты по порядку; пустой запрос — весь свод. */
    public List<Rule> search(String query) {
        String needle = query == null ? "" : query.trim().toLowerCase(Locale.ROOT);
        List<Rule> out = new ArrayList<>();
        for (Rule rule : rules) {
            if (needle.isEmpty()
                    || rule.code().toLowerCase(Locale.ROOT).contains(needle)
                    || rule.title().toLowerCase(Locale.ROOT).contains(needle)) {
                out.add(rule);
            }
        }
        out.sort(Comparator.comparing(Rule::code));
        return out;
    }

    public Rule byId(String id) {
        for (Rule rule : rules) {
            if (rule.id().equals(id)) {
                return rule;
            }
        }
        return null;
    }

    /** Что этот пункт разрешает: из них модератор и выбирает. */
    public List<Sanction> sanctionsOf(String ruleId) {
        List<Sanction> out = new ArrayList<>();
        for (Sanction sanction : sanctions) {
            if (sanction.rule_id().equals(ruleId)) {
                out.add(sanction);
            }
        }
        return out;
    }
}
