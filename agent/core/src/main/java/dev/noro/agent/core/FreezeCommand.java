package dev.noro.agent.core;

import java.util.Arrays;
import org.slf4j.Logger;

/**
 * Команды модерации {@code /freeze <player> [reason]} и {@code /unfreeze <player>}.
 */
public final class FreezeCommand {

    private final MasterClient master;
    private final Moderation moderation;
    private final Logger log;

    public FreezeCommand(MasterClient master, Moderation moderation, Logger log) {
        this.master = master;
        this.moderation = moderation;
        this.log = log;
    }

    public void freeze(CommandSender sender, String[] args, String lang) {
        if (!sender.console() && !sender.has("noro.mod.freeze")) {
            sender.reply(AgentStrings.get(lang, "no_perm_freeze", "#f87171You cannot freeze players."));
            return;
        }
        if (args.length < 1) {
            sender.reply("#f87171Usage: /freeze <player> [reason]");
            return;
        }
        String targetName = args[0];
        String reason = args.length > 1
                ? String.join(" ", Arrays.copyOfRange(args, 1, args.length))
                : "Suspicion of cheating";

        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, targetName);
            if (target == null) return;
            try {
                moderation.master().http().post("/api/admin/freezes", new FreezePayload(target.uuid(), reason));
                sender.reply(AgentStrings.get(lang, "player_frozen", "#4ade80Player {0} has been frozen.", target.username()));
                moderation.onProfileChanged(target.uuid());
            } catch (Exception e) {
                log.warn("Cannot freeze player {}: {}", target.username(), e.getMessage());
                sender.reply("#f87171Failed to freeze player.");
            }
        });
    }

    public void unfreeze(CommandSender sender, String[] args, String lang) {
        if (!sender.console() && !sender.has("noro.mod.freeze")) {
            sender.reply(AgentStrings.get(lang, "no_perm_freeze", "#f87171You cannot unfreeze players."));
            return;
        }
        if (args.length < 1) {
            sender.reply("#f87171Usage: /unfreeze <player>");
            return;
        }
        String targetName = args[0];
        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, targetName);
            if (target == null) return;
            try {
                moderation.master().http().delete("/api/admin/freezes/" + target.uuid());
                sender.reply(AgentStrings.get(lang, "player_unfrozen", "#4ade80Player {0} has been unfrozen.", target.username()));
                moderation.onProfileChanged(target.uuid());
            } catch (Exception e) {
                log.warn("Cannot unfreeze player {}: {}", target.username(), e.getMessage());
                sender.reply("#f87171Failed to unfreeze player.");
            }
        });
    }

    private record FreezePayload(java.util.UUID target, String reason) {}
}
