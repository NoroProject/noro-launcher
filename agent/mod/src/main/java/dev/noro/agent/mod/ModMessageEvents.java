package dev.noro.agent.mod;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Proxy;
import net.minecraft.server.level.ServerPlayer;

/**
 * Динамическая подписка на Fabric Message API (Styled Chat / Fabric API).
 *
 * <p>Использует Dynamic Proxy, чтобы компилироваться без зависимостей от Fabric API
 * на сборках NeoForge, но нативно глушить чат через Styled Chat во время игры.
 */
public final class ModMessageEvents {

    private ModMessageEvents() {}

    public static void register(AgentRuntime runtime) {
        try {
            Class<?> eventsClass = Class.forName("net.fabricmc.fabric.api.message.v1.ServerMessageEvents");
            Field allowChatField = eventsClass.getField("ALLOW_CHAT_MESSAGE");
            Object allowChatEvent = allowChatField.get(null);
            Method registerMethod = allowChatEvent.getClass().getMethod("register", Object.class);

            Class<?> listenerClass = Class.forName("net.fabricmc.fabric.api.message.v1.ServerMessageEvents$AllowChatMessage");
            Object proxy = Proxy.newProxyInstance(
                    listenerClass.getClassLoader(),
                    new Class<?>[] { listenerClass },
                    (p, method, args) -> {
                        if (args != null && args.length >= 2 && args[1] instanceof ServerPlayer sender) {
                            String text = args.length > 2 && args[2] != null ? args[2].toString() : "";
                            return !runtime.checkChatMessage(sender, text);
                        }
                        return true;
                    });
            registerMethod.invoke(allowChatEvent, proxy);
            AgentRuntime.LOG.info("Hooked into Fabric Message API / Styled Chat for mute handling");
        } catch (Throwable ignored) {
            // Fabric Message API отсутствует — работаем через стандарный ServerChatEvent
        }
    }
}
