package dev.noro.client.staff.ui;

import dev.noro.client.NoroCore;
import dev.noro.client.rules.RuleBook;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.network.chat.Component;

/**
 * Состояние формы наказания: правило задаёт рамки, модератор выбирает внутри.
 *
 * <p>Отдельно от экрана, потому что экран пересобирается на каждое изменение
 * размера окна, а выбранное правило и набранная причина — нет.
 */
final class PunishForm {

    private RuleBook.Rule rule;
    /** Выбранная вилка: по ней подсвечивается чип и берётся нижняя граница. */
    private RuleBook.Sanction sanction;

    private String kind = "warn";
    private String duration = "";
    private String reason = "";
    /** Код правила из дела: подставляется до того, как свод успел приехать. */
    private String pendingCode;

    void preselect(String ruleCode) {
        this.pendingCode = ruleCode;
    }

    RuleBook.Rule rule() {
        // Свод приезжает кадром, а экран мог открыться раньше: досматриваем
        // отложенный код, когда он наконец есть.
        if (rule == null && pendingCode != null && NoroCore.rules().loaded()) {
            for (RuleBook.Rule candidate : NoroCore.rules().search(pendingCode)) {
                if (candidate.code().equalsIgnoreCase(pendingCode)) {
                    pick(candidate);
                    break;
                }
            }
            pendingCode = null;
        }
        return rule;
    }

    /**
     * Выбрать пункт свода.
     *
     * <p>Причина набирается сама — её всё ещё можно дописать руками. Вид и срок
     * подставляются из первой вилки: она и есть «что за это полагается».
     */
    void pick(RuleBook.Rule picked) {
        if (picked == null) {
            return;
        }
        rule = picked;
        sanction = null;
        reason = "[" + picked.code() + "] " + picked.title();
        // Первая вилка — предложение по умолчанию: чаще всего оно и нужное.
        for (RuleBook.Sanction first : NoroCore.rules().sanctionsOf(picked.id())) {
            pick(first);
            break;
        }
    }

    /** Выбрать вилку: она задаёт вид и нижнюю границу срока. */
    void pick(RuleBook.Sanction picked) {
        sanction = picked;
        kind = picked.kind();
        duration = picked.min_minutes() == null ? "" : minutes(picked.min_minutes());
    }

    boolean picked(RuleBook.Sanction candidate) {
        return sanction != null
                && sanction.kind().equals(candidate.kind())
                && java.util.Objects.equals(sanction.min_minutes(), candidate.min_minutes());
    }

    /**
     * Виды, из которых можно выбирать.
     *
     * <p>Правило выбрано — только то, что оно разрешает: свод и существует,
     * чтобы за флуд не выдавали бан. Не выбрано — весь список, а рамки всё
     * равно проверит мастер.
     */
    List<String> kinds(List<String> all) {
        if (rule == null) {
            return all;
        }
        List<String> out = new ArrayList<>();
        for (RuleBook.Sanction s : NoroCore.rules().sanctionsOf(rule.id())) {
            if (!out.contains(s.kind())) {
                out.add(s.kind());
            }
        }
        return out.isEmpty() ? all : out;
    }

    void kind(String value) {
        this.kind = value;
    }

    String kind() {
        return kind;
    }

    String ruleCode() {
        RuleBook.Rule picked = rule();
        return picked == null ? null : picked.code();
    }

    String reason() {
        return reason;
    }

    String duration() {
        return duration;
    }

    void duration(String value) {
        this.duration = value;
    }

    /** Подпись кнопки — вид наказания: видно, что именно сейчас произойдёт. */
    Component applyLabel() {
        return Component.translatable("noro.cases.punish.kind." + kind);
    }

    static Component kindLabel(String kind) {
        return Component.translatable("noro.cases.punish.kind." + kind);
    }

    /** «Мут 15m–2h» — вилка одной строкой, как чип на сайте. */
    Component sanctionLabel(RuleBook.Sanction sanction) {
        Component kindName = Component.translatable("noro.cases.punish.kind." + sanction.kind());
        if (sanction.min_minutes() == null && sanction.max_minutes() == null) {
            return kindName;
        }
        String from = sanction.min_minutes() == null ? "0" : minutes(sanction.min_minutes());
        String to = sanction.max_minutes() == null ? "∞" : minutes(sanction.max_minutes());
        return Component.literal(kindName.getString() + " " + from + "–" + to);
    }

    /** Минуты человеческой строкой: 90 → «1h 30m», 1440 → «1d». */
    static String minutes(long total) {
        long days = total / (60 * 24);
        long hours = total % (60 * 24) / 60;
        long mins = total % 60;
        StringBuilder out = new StringBuilder();
        if (days > 0) {
            out.append(days).append('d');
        }
        if (hours > 0) {
            out.append(out.isEmpty() ? "" : " ").append(hours).append('h');
        }
        if (mins > 0 || out.isEmpty()) {
            out.append(out.isEmpty() ? "" : " ").append(mins).append('m');
        }
        return out.toString();
    }
}
