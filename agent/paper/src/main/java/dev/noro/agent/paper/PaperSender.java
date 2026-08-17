package dev.noro.agent.paper;

import dev.noro.agent.core.CommandSender;
import dev.noro.agent.core.NoroAgentApi;
import dev.noro.agent.core.PermissionSet;
import dev.noro.agent.core.PlayerProfile;
import java.util.UUID;
import net.kyori.adventure.text.serializer.legacy.LegacyComponentSerializer;
import org.bukkit.entity.Player;

/**
 * Тот, кто набрал команду, — глазами Bukkit.
 *
 * <p>Права спрашиваются у профиля мастера, а не у Bukkit: attachment выдаёт
 * только те узлы, которые кто-то зарегистрировал на сервере, и шаблон
 * {@code noro.mod.*} у игрока без явного {@code noro.mod.punish.ban} в реестре
 * не сработал бы. Мастер же присылает набор целиком.
 */
final class PaperSender implements CommandSender {

    private final org.bukkit.command.CommandSender sender;

    PaperSender(org.bukkit.command.CommandSender sender) {
        this.sender = sender;
    }

    @Override
    public UUID uuid() {
        return sender instanceof Player player ? player.getUniqueId() : null;
    }

    @Override
    public String name() {
        return sender.getName();
    }

    @Override
    public boolean has(String permission) {
        UUID uuid = uuid();
        if (uuid == null) {
            return true;
        }
        PlayerProfile profile = NoroAgentApi.cache().get(uuid);
        // Профиля нет — мастер был недоступен на входе. Тогда прав не знаем, а
        // «не знаем» для модерации значит «нельзя».
        return profile != null && PermissionSet.of(profile.permissions()).has(permission);
    }

    @Override
    public void reply(String message) {
        sender.sendMessage(LegacyComponentSerializer.legacySection().deserialize(message));
    }
}
