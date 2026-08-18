package dev.noro.agent.core;

import java.util.List;

/**
 * Команда {@code /rules [код]}: просмотр свода правил и конкретного пункта.
 */
public final class RuleCommands {

    private final RuleCatalog catalog;

    public RuleCommands(RuleCatalog catalog) {
        this.catalog = catalog;
    }

    public void rules(CommandSender sender, String[] args, String lang) {
        if (args.length == 0 || args[0].isBlank()) {
            list(sender, lang);
            return;
        }
        show(sender, args[0], lang);
    }

    private void list(CommandSender sender, String lang) {
        List<String> codes = catalog.matching("");
        if (codes.isEmpty()) {
            sender.reply("#fde047No rules loaded.");
            return;
        }
        sender.reply(AgentStrings.get(lang, "rules_title", codes.size()));
        for (String code : codes) {
            String clean = code.startsWith("@") ? code.substring(1) : code;
            String title = catalog.titleFor(clean);
            sender.reply(" #9ca3af• #ffffff" + clean + " #9ca3af— " + (title == null ? "" : title));
        }
    }

    private void show(CommandSender sender, String rawCode, String lang) {
        String clean = rawCode.startsWith("@") ? rawCode.substring(1) : rawCode.strip();
        String title = catalog.titleFor(clean);
        if (title == null) {
            sender.reply(AgentStrings.get(lang, "rule_not_found", clean));
            return;
        }
        sender.reply("#9ca3afRule #ffffff" + clean + "#9ca3af: #ffffff" + title);
        String reason = catalog.punishReasonFor(clean);
        if (reason != null && !reason.isBlank()) {
            sender.reply(" #9ca3af• #9ca3afReason: #ffffff" + reason);
        }
        List<String> durations = catalog.durationsFor(clean);
        if (!durations.isEmpty()) {
            sender.reply(" #9ca3af• #9ca3afDurations: #fde047" + String.join(", ", durations));
        }
    }
}
