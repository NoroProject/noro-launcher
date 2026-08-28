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
 * The rule book, shared by all Noro mods.
 *
 * <p>Public on purpose: every ban cites a rule and the banned player has to be
 * able to read it, so no permission is required to fetch this.
 */
public final class RuleBook implements Feature {

    public record Rule(String id, String category_id, String code, String title,
                       String description, int sort_order) {}

    /** What a rule allows as punishment, and within which bounds. */
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

    /** Refetch. The book changes rarely, so this is on demand rather than polled. */
    public void request() {
        if (bridge != null) {
            bridge.send(new RequestRules());
        }
    }

    public record RequestRules() {}

    public boolean loaded() {
        return !rules.isEmpty();
    }

    /** An empty query returns the whole book. */
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

    /** The set a moderator picks from for this rule. */
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
