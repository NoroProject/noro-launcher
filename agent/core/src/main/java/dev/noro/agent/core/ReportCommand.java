package dev.noro.agent.core;

import java.util.Arrays;
import java.util.function.Supplier;
import org.slf4j.Logger;

/**
 * Обработка команды {@code /report <player> <reason>}.
 *
 * <p>Место жалобы снимается здесь и сейчас: через пять минут игрок уже ушёл, а
 * гриф остался — и телепорт «к месту происшествия» в разборе ведёт туда, где
 * что-то произошло, а не туда, где стоит жалобщик, когда дело наконец взяли.
 */
public final class ReportCommand {

    private final MasterClient master;
    /**
     * Мост берётся в момент выполнения, а не в конструкторе.
     *
     * <p>Дерево команд строится на старте сервера, а мост появляется вместе с
     * каналом — позже. Спросить его сразу значит уронить регистрацию команд
     * целиком, а с ней и весь запуск.
     */
    private final Supplier<GameBridge> bridge;

    private final Logger log;

    public ReportCommand(MasterClient master, Supplier<GameBridge> bridge, Logger log) {
        this.master = master;
        this.bridge = bridge;
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
            // Позиции нет — канал ещё не поднялся, платформа её не отдаёт или
            // игрок уже вышел. Едет пустота, а не нули: «0 0 0» это точка в
            // мире, и модератор поедет именно туда, приняв её за место.
            GameBridge game = bridge.get();
            GameBridge.Position at =
                    game == null ? null : game.position(sender.uuid()).orElse(null);
            if (at == null) {
                log.warn("позиция жалобщика {} недоступна — жалоба уйдёт без места", sender.uuid());
            }
            master.createReport(sender.uuid(), target.uuid(), reason, at);
            sender.reply(AgentStrings.get(lang, "report_submitted", "#4ade80Report submitted against {0}.", target.username()));
        });
    }
}
