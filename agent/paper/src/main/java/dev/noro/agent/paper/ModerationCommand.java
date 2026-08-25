package dev.noro.agent.paper;

import dev.noro.agent.core.CheckCommand;
import dev.noro.agent.core.FreezeCommand;
import dev.noro.agent.core.GameBridge;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.Moderation;
import dev.noro.agent.core.ModerationCommands;
import dev.noro.agent.core.ReportCommand;
import dev.noro.agent.core.RuleCatalog;
import dev.noro.agent.core.RuleCommands;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.command.TabCompleter;

/**
 * Один обработчик на все команды модерации: разбор и логика живут в core, а
 * здесь остаётся сопоставление имени команды с видом наказания.
 */
final class ModerationCommand implements CommandExecutor, TabCompleter {

    /** Подсказки срока: то же, чем пользуются в панели. */
    private static final List<String> TERMS = List.of("30m", "1h", "6h", "1d", "7d", "30d", "perm");

    private final ModerationCommands commands;
    private final FreezeCommand freezeCmd;
    private final ReportCommand reportCmd;
    private final CheckCommand checkCmd;
    private final RuleCommands ruleCmds;
    private final GameBridge bridge;
    private final RuleCatalog rules;

    private final VanishManager vanishManager;
    private final Moderation moderation;

    ModerationCommand(MasterClient master, Moderation moderation, RuleCatalog rules, GameBridge bridge, VanishManager vanishManager, org.slf4j.Logger log) {
        this.moderation = moderation;
        this.commands = new ModerationCommands(master, moderation, log);
        this.freezeCmd = new FreezeCommand(master, moderation, log);
        this.reportCmd = new ReportCommand(master, () -> bridge, log);
        this.checkCmd = new CheckCommand(master, log);
        this.ruleCmds = new RuleCommands(rules);
        this.bridge = bridge;
        this.rules = rules;
        this.vanishManager = vanishManager;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        PaperSender actor = new PaperSender(sender);
        String lang = actor.console() ? "en" : null;
        switch (command.getName().toLowerCase(Locale.ROOT)) {
            case "ban" -> commands.punish(actor, "ban", args, lang);
            case "serverban" -> commands.punish(actor, "server_ban", args, lang);
            case "mute" -> commands.punish(actor, "mute", args, lang);
            case "warn" -> commands.punish(actor, "warn", args, lang);
            case "unban" -> commands.revoke(actor, "ban", args, lang);
            case "unmute" -> commands.revoke(actor, "mute", args, lang);
            case "history" -> commands.history(actor, args, lang);
            case "check" -> checkCmd.check(actor, args, lang);
            case "rules", "rule" -> ruleCmds.rules(actor, args, lang);
            case "freeze" -> freezeCmd.freeze(actor, args, lang);
            case "unfreeze" -> freezeCmd.unfreeze(actor, args, lang);
            case "report" -> reportCmd.execute(actor, args, lang);
            // Клик по меню разбора приходит сюда же, что и набранная команда.
            case "case" -> new dev.noro.agent.core.CaseCommands(
                            moderation.cases(), bridge, moderation.events(), freezeCmd)
                    .execute(actor, args, lang);
            case "vanish", "v" -> {
                if (sender instanceof org.bukkit.entity.Player p) {
                    if (args.length > 0 && args[0].equalsIgnoreCase("list")) {
                        vanishManager.sendVanishList(p);
                    } else if (p.hasPermission("noro.mod.vanish.use")) {
                        vanishManager.toggleVanish(p, lang);
                    } else {
                        p.sendMessage(dev.noro.agent.core.AgentStrings.get(lang, "no_perm_view"));
                    }
                }
            }
            default -> {
                return false;
            }
        }
        return true;
    }

    @Override
    public List<String> onTabComplete(CommandSender sender, Command command, String label, String[] args) {
        String name = command.getName().toLowerCase(Locale.ROOT);
        if (name.equals("rules") || name.equals("rule")) {
            if (args.length == 1) {
                return rules.matching(args[0].replaceFirst("^@", ""));
            }
            return List.of();
        }
        if (args.length == 1) {
            return prefixed(bridge.onlineNames(), args[0]);
        }
        if (name.equals("unban") || name.equals("unmute") || name.equals("history") || name.equals("check") || name.equals("freeze") || name.equals("unfreeze") || name.equals("report") || name.equals("vanish") || name.equals("v")) {
            return List.of();
        }
        // Второй аргумент — срок, если он у команды есть и ещё не назван.
        // Дальше идёт правило: до причины подсказывать больше нечего.
        boolean timed = !name.equals("warn");
        if (args.length == 2 && timed) {
            return prefixed(TERMS, args[1]);
        }
        if (args.length <= 3) {
            return rules.matching(args[args.length - 1].replaceFirst("^@", ""));
        }
        return List.of();
    }

    private static List<String> prefixed(Iterable<String> options, String typed) {
        String lower = typed.toLowerCase(Locale.ROOT);
        List<String> out = new ArrayList<>();
        for (String option : options) {
            if (option.toLowerCase(Locale.ROOT).startsWith(lower)) {
                out.add(option);
            }
        }
        return out;
    }

    /** Имена, которые надо объявить в {@code plugin.yml}. */
    static List<String> names() {
        return List.of("ban", "serverban", "mute", "warn", "unban", "unmute", "history", "check", "rules", "rule", "freeze", "unfreeze", "report", "vanish", "v", "case");
    }
}
