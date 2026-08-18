package dev.noro.agent.core;

import java.util.List;
import org.slf4j.Logger;

/**
 * Команда {@code /check <ник>}: карточка игрока (роли, баны, муты, заморозка, варны).
 */
public final class CheckCommand {

    private final MasterClient master;
    private final Logger log;

    public CheckCommand(MasterClient master, Logger log) {
        this.master = master;
        this.log = log;
    }

    public void check(CommandSender sender, String[] args, String lang) {
        if (!sender.console() && !sender.has("noro.mod.punish.view")) {
            sender.reply(AgentStrings.get(lang, "no_perm_view"));
            return;
        }
        if (args.length < 1 || args[0].isBlank()) {
            sender.reply("#f87171Usage: /check <player>");
            return;
        }
        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, args[0]);
            if (target == null) {
                return;
            }
            sender.reply(AgentStrings.get(lang, "check_card_title", target.username()));
            sender.reply(AgentStrings.get(lang, "check_card_roles", formatRoles(target)));
            if (target.freezeInfo() != null) {
                sender.reply(AgentStrings.get(lang, "check_card_freeze", "#f87171[FROZEN] " + target.freezeInfo().reason() + " (" + target.freezeInfo().frozenBy() + ")"));
            }
            if (target.activeMute() != null) {
                sender.reply(AgentStrings.get(lang, "check_card_mute", target.activeMute().reason()));
            }
            if (target.activeBan() != null) {
                sender.reply(AgentStrings.get(lang, "check_card_ban", target.activeBan().reason()));
            }
            sender.reply(AgentStrings.get(lang, "check_card_warns", target.pendingWarns().size()));
        });
    }

    private static String formatRoles(PlayerProfile profile) {
        if (profile.roles() == null || profile.roles().isEmpty()) {
            return "default";
        }
        List<String> names = profile.roles().stream()
                .map(r -> r.displayName() != null && !r.displayName().isBlank() ? r.displayName() : r.name())
                .toList();
        return String.join(", ", names);
    }
}
