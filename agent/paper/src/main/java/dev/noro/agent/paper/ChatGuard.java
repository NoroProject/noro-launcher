package dev.noro.agent.paper;

import dev.noro.agent.core.ChatCommands;
import dev.noro.agent.core.Moderation;
import io.papermc.paper.event.player.AsyncChatEvent;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerCommandPreprocessEvent;

/**
 * Мут в действии: замученному не даём ни писать в чат, ни говорить командой.
 *
 * <p>{@code LOWEST} и без {@code ignoreCancelled}: сообщение надо отсечь до
 * того, как его увидит чат-плагин или лог. Отказ при этом показывается всегда —
 * молча съеденное сообщение выглядит как поломка сервера, и игрок повторяет
 * его снова и снова.
 */
final class ChatGuard implements Listener {

    private final Moderation moderation;

    ChatGuard(Moderation moderation) {
        this.moderation = moderation;
    }

    @EventHandler(priority = EventPriority.LOWEST)
    public void onChat(AsyncChatEvent event) {
        deny(event.getPlayer(), event);
    }

    @EventHandler(priority = EventPriority.LOWEST)
    public void onCommand(PlayerCommandPreprocessEvent event) {
        if (ChatCommands.speaks(event.getMessage())) {
            deny(event.getPlayer(), event);
        }
    }

    private void deny(Player player, org.bukkit.event.Cancellable event) {
        // Короткая версия: над хотбаром одна строка, длинный чат-текст там
        // обрезается и читается как мусор.
        String notice = moderation.muteNotice(player.getUniqueId(), player.getName(), true);
        if (notice != null) {
            event.setCancelled(true);
            player.sendActionBar(PaperText.parse(notice));
        }
    }
}
