package dev.noro.agent.core;

import java.util.Arrays;
import org.slf4j.Logger;

/**
 * Обработка команды {@code /report <player> <reason>}.
 */
public final class ReportCommand {

    private final MasterClient master;
    private final Logger log;

    public ReportCommand(MasterClient master, Logger log) {
        this.master = master;
        this.log = log;
    }

    public void execute(CommandSender sender, String[] args, String lang) {
        if (args.length < 2) {
            sender.reply(AgentStrings.get(lang, "report_usage", "Usage: /report <player> <reason>"));
            return;
        }
        String targetName = args[0];
        String reason = String.join(" ", Arrays.copyOfRange(args, 1, args.length));

        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, targetName);
            if (target == null) {
                return;
            }
            if (sender.uuid() == null) {
                sender.reply("#f87171Console cannot report players.");
                return;
            }
            master.createReport(sender.uuid(), target.uuid(), reason);
            sender.reply(AgentStrings.get(lang, "report_submitted", "#4ade80Report submitted against {0}.", target.username()));
        });
    }
}
