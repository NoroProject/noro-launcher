//#if FABRIC
package dev.noro.agent.mod;

import net.fabricmc.api.DedicatedServerModInitializer;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import net.fabricmc.fabric.api.networking.v1.ServerPlayConnectionEvents;
import net.fabricmc.loader.api.FabricLoader;

/**
 * Точка входа Fabric. Весь файл под {@code //#if FABRIC}: на сборке NeoForge он
 * целиком закомментируется и класса просто не будет.
 *
 * <p>{@code DedicatedServerModInitializer}, а не общий {@code ModInitializer}:
 * агенту нечего делать на клиенте.
 */
public final class FabricEntry implements DedicatedServerModInitializer {

    private final AgentRuntime runtime = new AgentRuntime();

    @Override
    public void onInitializeServer() {
        if (!runtime.init(FabricLoader.getInstance().getConfigDir())) {
            return;
        }
        ServerLifecycleEvents.SERVER_STARTED.register(runtime::onServerStarted);
        ServerLifecycleEvents.SERVER_STOPPING.register(server -> runtime.onServerStopping());
        ServerPlayConnectionEvents.JOIN.register(
                (handler, sender, server) -> runtime.onPlayerJoin(handler.player));
        ServerPlayConnectionEvents.DISCONNECT.register(
                (handler, server) -> runtime.onPlayerLeave(handler.player.getUUID()));

        // CommandBuildContext появился в 1.19 — до него у колбэка два аргумента.
        // Пакет колбэка тоже разный: v2 живёт только с 1.19.
        //#if MC>=11900
        net.fabricmc.fabric.api.command.v2.CommandRegistrationCallback.EVENT.register(
                (dispatcher, registryAccess, environment) -> runtime.registerCommands(dispatcher));
        //#else
        //$$ net.fabricmc.fabric.api.command.v1.CommandRegistrationCallback.EVENT.register(
        //$$         (dispatcher, dedicated) -> runtime.registerCommands(dispatcher));
        //#endif

        // Мут держится на fabric-message-api, а он появился в 1.19.1 вместе с
        // подписанным чатом. На 1.18 и 1.19 перехватить чат без миксина нечем,
        // поэтому там мут доезжает только запретом на вход командой — и это
        // честнее, чем тянуть свой миксин в каждую из тридцати сборок.
        //#if MC>=11901
        net.fabricmc.fabric.api.message.v1.ServerMessageEvents.ALLOW_CHAT_MESSAGE.register(
                (message, sender, params) -> !runtime.silenced(sender));
        net.fabricmc.fabric.api.message.v1.ServerMessageEvents.ALLOW_COMMAND_MESSAGE.register(
                (message, source, params) -> !(source.getEntity() instanceof net.minecraft.server.level.ServerPlayer p)
                        || !runtime.silenced(p));
        //#endif
    }
}
//#endif
