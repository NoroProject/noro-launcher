package dev.noro.agent.mod;

import com.mojang.brigadier.context.CommandContext;
import com.mojang.brigadier.suggestion.Suggestions;
import com.mojang.brigadier.suggestion.SuggestionsBuilder;
import dev.noro.agent.core.DurationArg;
import dev.noro.agent.core.ModerationCommands;
import dev.noro.agent.core.RuleCatalog;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.CompletableFuture;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.SharedSuggestionProvider;

/**
 * Подсказки внутри жадной строки команды наказания.
 *
 * <p>Brigadier подсказывает по узлам, а у нас после ника идёт одна строка
 * целиком: срок, код правила и причина. Поэтому позицию слова приходится
 * считать самим — зато модератор получает подсказку там, где печатает.
 *
 * <p>Что предлагается: коды правил, сроки (сначала рамки из свода — мастер всё
 * равно откажет за их пределами) и готовая формулировка причины по правилу.
 */
final class ModSuggestions {

    /** Ходовые сроки. Идут после рамок правила, если те известны. */
    private static final List<String> COMMON = List.of("10m", "30m", "1h", "12h", "1d", "7d", "30d", "perm");

    private final ModerationCommands commands;
    private final RuleCatalog rules;

    ModSuggestions(ModerationCommands commands, RuleCatalog rules) {
        this.commands = commands;
        this.rules = rules;
    }

    CompletableFuture<Suggestions> suggest(
            CommandContext<CommandSourceStack> context, SuggestionsBuilder builder, boolean timed) {
        String typed = builder.getRemaining();
        int lastSpace = typed.lastIndexOf(' ');
        int wordStart = lastSpace < 0 ? 0 : lastSpace + 1;
        String word = typed.substring(wordStart);
        String[] parts = typed.stripLeading().split("\\s+");
        int done = lastSpace < 0 ? 0 : (typed.endsWith(" ") ? parts.length : parts.length - 1);

        List<String> out = new ArrayList<>();
        if (done == 0) {
            out.addAll(rules.matching(word));
            if (timed && !word.startsWith("@")) {
                addMatching(out, COMMON, word);
            }
        } else if (timed && done == 1) {
            secondWord(out, parts[0], word);
        } else {
            reason(out, parts, done, word);
        }
        return SharedSuggestionProvider.suggest(out, builder.createOffset(builder.getStart() + wordStart));
    }

    /** Первым словом был срок — дальше правило, и наоборот. */
    private void secondWord(List<String> out, String first, String word) {
        if (DurationArg.parse(first) != null) {
            out.addAll(rules.matching(word));
            return;
        }
        addMatching(out, rules.durationsFor(first), word);
        if (!word.startsWith("@")) {
            addMatching(out, COMMON, word);
        }
    }

    /** Дальше идёт причина: предлагаем ту же, что уедет мастеру сама. */
    private void reason(List<String> out, String[] parts, int done, String word) {
        String rule = null;
        for (int i = 0; i < done && i < parts.length; i++) {
            rule = ruleCode(parts[i]);
            if (rule != null) {
                break;
            }
        }
        if (rule == null) {
            return;
        }
        String suggested = commands.suggestedReason(rule);
        if (suggested != null && !suggested.isBlank()) {
            addMatching(out, List.of(suggested), word);
        }
    }

    private static void addMatching(List<String> out, List<String> options, String word) {
        String lower = word.toLowerCase(Locale.ROOT);
        for (String option : options) {
            if (!out.contains(option) && option.toLowerCase(Locale.ROOT).startsWith(lower)) {
                out.add(option);
            }
        }
    }

    /** Код пункта: `1.1` или `@chat.spam`. */
    private static String ruleCode(String text) {
        if (text == null || text.length() < 2) {
            return null;
        }
        String raw = text.startsWith("@") ? text.substring(1) : text;
        return raw.matches("[A-Za-z0-9_]+(\\.[A-Za-z0-9_]+)+") ? raw : null;
    }
}
