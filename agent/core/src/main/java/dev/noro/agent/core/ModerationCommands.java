package dev.noro.agent.core;

import java.util.List;
import org.slf4j.Logger;

/**
 * Команды модерации, одинаковые на всех платформах.
 */
public final class ModerationCommands {

    public static final List<String> TIMED = List.of("ban", "server_ban", "mute");

    private final MasterClient master;
    private final Moderation moderation;
    private final Logger log;

    public ModerationCommands(MasterClient master, Moderation moderation, Logger log) {
        this.master = master;
        this.moderation = moderation;
        this.log = log;
    }

    public String suggestedReason(String ruleCode) {
        return moderation.reasonForRule(ruleCode);
    }

    public static String permission(String kind) {
        return "noro.mod.punish." + kind;
    }

    public void punish(CommandSender sender, String kind, String[] args) {
        punish(sender, kind, args, null);
    }

    public void punish(CommandSender sender, String kind, String[] args, String lang) {
        if (!sender.console() && !sender.has(permission(kind))) {
            sender.reply(AgentStrings.get(lang, "no_perm_punish", kind));
            return;
        }
        PunishArgs parsed = PunishArgs.parse(args, TIMED.contains(kind));
        if (parsed == null) {
            sender.reply(PunishArgs.usage(command(kind), TIMED.contains(kind)));
            return;
        }
        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, parsed.target());
            if (target == null) return;
            String reason = parsed.reason().isBlank()
                    ? moderation.reasonForRule(parsed.ruleCode())
                    : parsed.reason();
            PunishmentInfo issued = moderation.client().punish(
                    target.uuid(), kind, reason, parsed.minutes(), parsed.ruleCode(), sender.uuid());
            sender.reply(moderation.render(moderation.templates().actorReceipt(), issued, target.username()));
            moderation.applier().apply(target.uuid(), target.username(), issued);
        });
    }

    public void revoke(CommandSender sender, String kind, String[] args) {
        revoke(sender, kind, args, null);
    }

    public void revoke(CommandSender sender, String kind, String[] args, String lang) {
        if (!sender.console() && !sender.has("noro.mod.punish.revoke")) {
            sender.reply(AgentStrings.get(lang, "no_perm_revoke"));
            return;
        }
        if (args.length < 1) {
            sender.reply("§cUsage: /un" + command(kind) + " <player>");
            return;
        }
        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, args[0]);
            if (target == null) return;
            int lifted = 0;
            for (PunishmentInfo punishment : moderation.client().history(target.uuid())) {
                if (!punishment.active() || !matches(punishment.kind(), kind)) continue;
                moderation.client().revoke(punishment.id(), sender.uuid());
                lifted++;
            }
            if (lifted == 0) {
                sender.reply(AgentStrings.get(lang, "no_active_punishment", target.username(), kind));
                return;
            }
            sender.reply(AgentStrings.get(lang, "punishment_lifted", lifted, kind, target.username()));
            moderation.applier().lift(target.uuid(), kind);
        });
    }

    public void history(CommandSender sender, String[] args) {
        history(sender, args, null);
    }

    public void history(CommandSender sender, String[] args, String lang) {
        if (!sender.console() && !sender.has("noro.mod.punish.view")) {
            sender.reply(AgentStrings.get(lang, "no_perm_view"));
            return;
        }
        if (args.length < 1) {
            sender.reply("§cUsage: /history <player>");
            return;
        }
        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, args[0]);
            if (target == null) return;
            List<PunishmentInfo> rows = moderation.client().history(target.uuid());
            if (rows.isEmpty()) {
                sender.reply(AgentStrings.get(lang, "clean_record", target.username()));
                return;
            }
            sender.reply(AgentStrings.get(lang, "history_title", target.username()));
            for (PunishmentInfo row : rows.subList(0, Math.min(rows.size(), 10))) {
                sender.reply(line(row));
            }
        });
    }

    private static boolean matches(String punishmentKind, String requested) {
        return "ban".equals(requested)
                ? "ban".equals(punishmentKind) || "server_ban".equals(punishmentKind)
                : requested.equals(punishmentKind);
    }

    private static String command(String kind) {
        return "server_ban".equals(kind) ? "serverban" : kind;
    }

    private static String line(PunishmentInfo row) {
        String state = row.active() ? "§c" + row.kind() : "§8" + row.kind();
        String term = !row.active() ? "over" : DurationArg.remaining(row.left());
        return state + " §7" + term + " §fby " + row.actorLabel() + " §7— " + row.reason();
    }
}

