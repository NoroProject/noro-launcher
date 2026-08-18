package dev.noro.agent.mod;

import dev.noro.agent.core.AgentEvents;
import java.util.UUID;
import net.minecraft.server.level.ServerPlayer;

/**
 * Кто вошёл и кто вышел — наверх, мастеру.
 *
 * <p>Отдельно от {@link AgentRuntime}: тот про жизненный цикл мода, а это про
 * то, что мастер узнаёт о живом сервере. Смешивать их значит держать работу с
 * адресами игрока рядом с загрузкой конфигурации.
 *
 * <p>Канала может не быть — сервер стартует раньше, чем тот поднимется, и
 * рвётся он когда угодно. Потеря кадра допустима: состав мастер сверяет по
 * heartbeat, и событие лишь ускоряет сходимость.
 *
 * <p>Адрес игрока отсюда не уходит. Достать его одинаково на всём диапазоне
 * нечем: до 1.20.2 геттера у слушателя не было, а поле соединения на Fabric
 * закрыто маппингами — и это разошлось бы на директивы по лоадеру и по версии
 * ради поля, которого пока никто не спрашивает. На Paper адрес есть, там он и
 * отправляется. Вернуться к этому надо вместе с сессиями (п.6).
 */
final class ModPresence {

    private final ModModeration moderation;

    ModPresence(ModModeration moderation) {
        this.moderation = moderation;
    }

    void joined(ServerPlayer player) {
        AgentEvents events = moderation.events();
        if (events != null) {
            boolean vanished = ModVanishManager.getInstance().isVanished(player.getUUID());
            events.playerJoin(player.getUUID(), null, vanished);
        }
    }

    void left(UUID uuid) {
        AgentEvents events = moderation.events();
        if (events != null) {
            events.playerLeave(uuid, "quit");
        }
    }
}
