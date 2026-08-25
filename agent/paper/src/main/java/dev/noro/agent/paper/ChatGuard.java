package dev.noro.agent.paper;

import dev.noro.agent.core.ChatCommands;
import dev.noro.agent.core.Moderation;
import dev.noro.agent.core.automod.ChatFilters;
import io.papermc.paper.event.player.AsyncChatEvent;
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerCommandPreprocessEvent;

/**
 * Мут и автомодерация в действии: отсекаем спам, капс, рекламу и мат до рассылки в чат.
 */
final class ChatGuard implements Listener {

    private final Moderation moderation;

    ChatGuard(Moderation moderation) {
        this.moderation = moderation;
    }

    @EventHandler(priority = EventPriority.LOWEST)
    public void onChat(AsyncChatEvent event) {
        Player player = event.getPlayer();
        if (denyMute(player, event)) {
            return;
        }

        String plainText = PlainTextComponentSerializer.plainText().serialize(event.message());
        // Буфер держит окно разговора: срез из него уедет в дело, когда
        // появится повод. Пишем до фильтров — заблокированная реплика как раз
        // и есть то, что интересно разбору.
        moderation.chatRing().message(player.getUniqueId(), player.getName(), "public", plainText);

        ChatFilters.Result res = moderation.checkChatMessage(player.getUniqueId(), plainText);
        if (res.action() == ChatFilters.Action.DENY || res.action() == ChatFilters.Action.PUNISH || res.action() == ChatFilters.Action.ESCALATE) {
            event.setCancelled(true);
            player.sendActionBar(PaperText.parse("#f87171Message blocked by AutoMod [" + res.filterType() + "]"));
        }
    }

    @EventHandler(priority = EventPriority.LOWEST)
    public void onCommand(PlayerCommandPreprocessEvent event) {
        Player player = event.getPlayer();
        moderation.chatRing().command(player.getUniqueId(), player.getName(), event.getMessage());
        if (ChatCommands.speaks(event.getMessage())) {
            denyMute(event.getPlayer(), event);
        }
    }

    private boolean denyMute(Player player, org.bukkit.event.Cancellable event) {
        String notice = moderation.muteNotice(player.getUniqueId(), player.getName(), true);
        if (notice != null) {
            event.setCancelled(true);
            player.sendActionBar(PaperText.parse(notice));
            return true;
        }
        return false;
    }
}
