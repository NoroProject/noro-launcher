package dev.noro.agent.core;

import java.util.List;
import org.slf4j.Logger;

/**
 * Команды модерации, одинаковые на всех трёх платформах.
 *
 * <p>Платформенный модуль только регистрирует имена и передаёт сюда аргументы:
 * разбор, права, запрос к мастеру и ответ модератору живут здесь. Иначе
 * {@code /ban} на Paper и на Fabric разошлись бы в поведении на первой же
 * правке.
 */
public final class ModerationCommands {

    /** Виды наказаний, у которых есть срок. Предупреждение выдаётся навсегда. */
    public static final List<String> TIMED = List.of("ban", "server_ban", "mute");

    private final MasterClient master;
    private final Moderation moderation;
    private final Logger log;

    public ModerationCommands(MasterClient master, Moderation moderation, Logger log) {
        this.master = master;
        this.moderation = moderation;
        this.log = log;
    }

    /**
     * Готовая причина по пункту свода — для подсказки в командной строке.
     *
     * <p>Тот же текст, который уедет мастеру, если модератор ничего не напишет:
     * подсказка и результат обязаны совпадать, иначе она сбивает с толку.
     */
    public String suggestedReason(String ruleCode) {
        return moderation.reasonForRule(ruleCode);
    }

    /** Право на вид наказания — то же, что проверит мастер. */
    public static String permission(String kind) {
        return "noro.mod.punish." + kind;
    }

    /** {@code /ban}, {@code /serverban}, {@code /mute}, {@code /warn}. */
    public void punish(CommandSender sender, String kind, String[] args) {
        if (!sender.console() && !sender.has(permission(kind))) {
            sender.reply("§cYou cannot issue a " + kind + ".");
            return;
        }
        PunishArgs parsed = PunishArgs.parse(args, TIMED.contains(kind));
        if (parsed == null) {
            sender.reply(PunishArgs.usage(command(kind), TIMED.contains(kind)));
            return;
        }
        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, parsed.target());
            if (target == null) {
                return;
            }
            // Причина по правилу собирается из шаблона мастера: модератор
            // назвал пункт и не стал ничего дописывать.
            String reason = parsed.reason().isBlank()
                    ? moderation.reasonForRule(parsed.ruleCode())
                    : parsed.reason();
            PunishmentInfo issued = moderation
                    .client()
                    .punish(
                            target.uuid(),
                            kind,
                            reason,
                            parsed.minutes(),
                            parsed.ruleCode(),
                            sender.uuid());
            sender.reply(moderation.render(moderation.templates().actorReceipt(), issued, target.username()));
            // Мастер разошлёт это и по живому каналу, но ждать оттуда нельзя:
            // канал мог отвалиться, а наказание уже выдано.
            moderation.applier().apply(target.uuid(), target.username(), issued);
        });
    }

    /** {@code /unban}, {@code /unmute}: снимает все действующие такого вида. */
    public void revoke(CommandSender sender, String kind, String[] args) {
        if (!sender.console() && !sender.has("noro.mod.punish.revoke")) {
            sender.reply("§cYou cannot lift punishments.");
            return;
        }
        if (args.length < 1) {
            sender.reply("§cUsage: /un" + command(kind) + " <player>");
            return;
        }
        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, args[0]);
            if (target == null) {
                return;
            }
            int lifted = 0;
            for (PunishmentInfo punishment : moderation.client().history(target.uuid())) {
                if (!punishment.active() || !matches(punishment.kind(), kind)) {
                    continue;
                }
                moderation.client().revoke(punishment.id(), sender.uuid());
                lifted++;
            }
            if (lifted == 0) {
                sender.reply("§e" + target.username() + " has no active " + kind + ".");
                return;
            }
            sender.reply("§aLifted " + lifted + " " + kind + " for " + target.username() + ".");
            moderation.applier().lift(target.uuid(), kind);
        });
    }

    /** {@code /history <player>} — последние наказания, включая снятые. */
    public void history(CommandSender sender, String[] args) {
        if (!sender.console() && !sender.has("noro.mod.punish.view")) {
            sender.reply("§cYou cannot see punishments.");
            return;
        }
        if (args.length < 1) {
            sender.reply("§cUsage: /history <player>");
            return;
        }
        CommandSupport.async(sender, log, () -> {
            PlayerProfile target = CommandSupport.target(master, sender, args[0]);
            if (target == null) {
                return;
            }
            List<PunishmentInfo> rows = moderation.client().history(target.uuid());
            if (rows.isEmpty()) {
                sender.reply("§a" + target.username() + " has a clean record.");
                return;
            }
            sender.reply("§7Punishments of §f" + target.username() + "§7:");
            // Десяти строк хватает на разбор: за старым идут в панель, где есть
            // и постраничность, и фильтры.
            for (PunishmentInfo row : rows.subList(0, Math.min(rows.size(), 10))) {
                sender.reply(line(row));
            }
        });
    }

    /** Бан сети снимается и командой {@code /unban}: для игрока это один бан. */
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
        // У снятого и истёкшего остаток считать нечего: там важно, что оно уже
        // не действует, а не сколько было бы осталось.
        String term = !row.active() ? "over" : DurationArg.remaining(row.left());
        return state + " §7" + term + " §fby " + row.actorLabel() + " §7— " + row.reason();
    }
}
