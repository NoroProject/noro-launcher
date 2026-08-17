package dev.noro.agent.mod;

import dev.noro.agent.core.CommandSender;
import dev.noro.agent.core.NoroAgentApi;
import dev.noro.agent.core.PermissionSet;
import dev.noro.agent.core.PlayerProfile;
import java.util.UUID;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;

/**
 * Тот, кто набрал команду, — глазами Brigadier.
 *
 * <p>Права спрашиваются у профиля мастера. Ванильный уровень оператора здесь
 * не годится: он один на все команды, а у нас у мута и бана разные права, и
 * выдаёт их мастер, а не {@code ops.json}.
 */
final class ModSender implements CommandSender {

    private final CommandSourceStack source;

    ModSender(CommandSourceStack source) {
        this.source = source;
    }

    @Override
    public UUID uuid() {
        // getEntity(), а не getPlayer(): последний до 1.19 бросал
        // CommandSyntaxException, и одинакового вызова на весь диапазон версий
        // из него не выходит.
        return source.getEntity() instanceof ServerPlayer player ? player.getUUID() : null;
    }

    @Override
    public String name() {
        return source.getTextName();
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
        Component component = text(message);
        //#if MC>=12000
        source.sendSuccess(() -> component, false);
        //#else
        //$$ source.sendSuccess(component, false);
        //#endif
    }

    private static Component text(String message) {
        //#if MC>=11900
        return Component.literal(message);
        //#else
        //$$ return new net.minecraft.network.chat.TextComponent(message);
        //#endif
    }
}
