//#if FABRIC
//$$ package dev.noro.agent.mod;
//$$
//$$ import net.fabricmc.api.DedicatedServerModInitializer;
//$$ import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
//$$ import net.fabricmc.fabric.api.networking.v1.ServerPlayConnectionEvents;
//$$ import net.fabricmc.loader.api.FabricLoader;
//$$
//$$ /**
//$$  * Точка входа Fabric. Весь файл под {@code //#if FABRIC}: на сборке NeoForge он
//$$  * целиком закомментируется и класса просто не будет.
//$$  *
//$$  * <p>{@code DedicatedServerModInitializer}, а не общий {@code ModInitializer}:
//$$  * агенту нечего делать на клиенте.
//$$  */
//$$ public final class FabricEntry implements DedicatedServerModInitializer {
//$$
//$$     private final AgentRuntime runtime = new AgentRuntime();
//$$
//$$     @Override
//$$     public void onInitializeServer() {
//$$         if (!runtime.init(FabricLoader.getInstance().getConfigDir())) {
//$$             return;
//$$         }
//$$         ServerLifecycleEvents.SERVER_STARTED.register(runtime::onServerStarted);
//$$         ServerLifecycleEvents.SERVER_STOPPING.register(server -> runtime.onServerStopping());
//$$         ServerPlayConnectionEvents.JOIN.register(
//$$                 (handler, sender, server) -> runtime.onPlayerJoin(handler.player));
//$$     }
//$$ }
//#endif
