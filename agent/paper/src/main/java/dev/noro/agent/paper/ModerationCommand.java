package dev.noro.agent.paper;

import dev.noro.agent.core.GameBridge;
import dev.noro.agent.core.ModerationCommands;
import dev.noro.agent.core.RuleCatalog;
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
    private final GameBridge bridge;
    private final RuleCatalog rules;

    ModerationCommand(ModerationCommands commands, GameBridge bridge, RuleCatalog rules) {
        this.commands = commands;
        this.bridge = bridge;
        this.rules = rules;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        PaperSender actor = new PaperSender(sender);
        switch (command.getName().toLowerCase(Locale.ROOT)) {
            case "ban" -> commands.punish(actor, "ban", args);
            case "serverban" -> commands.punish(actor, "server_ban", args);
            case "mute" -> commands.punish(actor, "mute", args);
            case "warn" -> commands.punish(actor, "warn", args);
            case "unban" -> commands.revoke(actor, "ban", args);
            case "unmute" -> commands.revoke(actor, "mute", args);
            case "history" -> commands.history(actor, args);
            default -> {
                return false;
            }
        }
        return true;
    }

    @Override
    public List<String> onTabComplete(CommandSender sender, Command command, String label, String[] args) {
        if (args.length == 1) {
            return prefixed(bridge.onlineNames(), args[0]);
        }
        String name = command.getName().toLowerCase(Locale.ROOT);
        if (name.equals("unban") || name.equals("unmute") || name.equals("history")) {
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
        return List.of("ban", "serverban", "mute", "warn", "unban", "unmute", "history");
    }
}
