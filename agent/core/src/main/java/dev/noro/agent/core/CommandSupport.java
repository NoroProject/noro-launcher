package dev.noro.agent.core;

import java.io.IOException;
import java.util.concurrent.CompletableFuture;
import org.slf4j.Logger;

/**
 * Общее для всех команд модерации: уход в фон и разговор об ошибках.
 *
 * <p>Сеть в игровом потоке — это подвисший сервер на каждую команду, поэтому
 * работа уходит в фон целиком. Отказ мастера при этом не ошибка агента, а
 * ответ модератору: «правило допускает только до 7d» надо показать дословно,
 * а не спрятать за «internal error» в консоли.
 */
final class CommandSupport {

    private CommandSupport() {}

    interface Action {
        void run() throws IOException, InterruptedException;
    }

    static void async(CommandSender sender, Logger log, Action action) {
        CompletableFuture.runAsync(() -> {
            try {
                action.run();
            } catch (MasterRefusedException e) {
                // С номером: причина приходит по-английски, а номер одинаков в
                // любом языке и читается со скриншота чата.
                sender.reply("§c" + e.display());
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            } catch (Exception e) {
                sender.reply("§cMaster is unreachable, nothing was saved.");
                log.warn("Moderation command failed: {}", e.getMessage());
            }
        });
    }

    /** Игрок по нику или внятный отказ. Ищет мастер: он знает и оффлайн. */
    static PlayerProfile target(MasterClient master, CommandSender sender, String name)
            throws IOException, InterruptedException {
        PlayerProfile profile = master.playerByName(name).orElse(null);
        if (profile == null) {
            sender.reply("§cNo player named " + name + " on this network.");
        }
        return profile;
    }
}
