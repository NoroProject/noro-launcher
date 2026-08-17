//#if NEOFORGE
//$$ package dev.noro.agent.mod;
//$$
//$$ import dev.noro.agent.core.ChatCommands;
//$$ import java.util.UUID;
//$$ import net.minecraft.server.level.ServerPlayer;
//$$ import net.neoforged.bus.api.SubscribeEvent;
//$$ import net.neoforged.fml.common.Mod;
//$$ import net.neoforged.fml.loading.FMLPaths;
//$$ import net.neoforged.neoforge.common.NeoForge;
//$$ import net.neoforged.neoforge.event.CommandEvent;
//$$ import net.neoforged.neoforge.event.RegisterCommandsEvent;
//$$ import net.neoforged.neoforge.event.ServerChatEvent;
//$$ import net.neoforged.neoforge.event.entity.player.PlayerEvent;
//$$ import net.neoforged.neoforge.event.entity.player.PlayerNegotiationEvent;
//$$ import net.neoforged.neoforge.event.server.ServerStartedEvent;
//$$ import net.neoforged.neoforge.event.server.ServerStoppingEvent;
//$$ import net.neoforged.neoforge.server.permission.events.PermissionGatherEvent;
//$$
//$$ /**
//$$  * Точка входа NeoForge. Весь файл под {@code //#if NEOFORGE}: на сборке Fabric
//$$  * он закомментирован и класса не существует.
//$$  */
//$$ @Mod("noro_agent")
//$$ public final class NeoForgeEntry {
//$$
//$$     private final AgentRuntime runtime = new AgentRuntime();
//$$
//$$     public NeoForgeEntry() {
//$$         if (!runtime.init(FMLPaths.CONFIGDIR.get())) {
//$$             return;
//$$         }
//$$         NeoForge.EVENT_BUS.register(this);
//$$     }
//$$
//$$     @SubscribeEvent
//$$     public void onServerStarted(ServerStartedEvent event) {
//$$         runtime.onServerStarted(event.getServer());
//$$         NoroPermissionHandler.warnIfInactive(AgentRuntime.LOG);
//$$     }
//$$
//$$     @SubscribeEvent
//$$     public void onServerStopping(ServerStoppingEvent event) {
//$$         runtime.onServerStopping();
//$$     }
//$$
//$$     /** Оба Gather-события приходят на старте сервера, до входа первого игрока. */
//$$     @SubscribeEvent
//$$     public void onGatherHandler(PermissionGatherEvent.Handler event) {
//$$         event.addPermissionHandler(
//$$                 NoroPermissionHandler.IDENTIFIER,
//$$                 nodes -> new NoroPermissionHandler(nodes, runtime.permissions()));
//$$     }
//$$
//$$     @SubscribeEvent
//$$     public void onGatherNodes(PermissionGatherEvent.Nodes event) {
//$$         runtime.permissions().rememberNodes(NoroPermissionHandler.names(event.getNodes()));
//$$     }
//$$
//$$     /**
//$$      * Единственная фаза, где можно сходить в сеть и всё же успеть: логин ждёт
//$$      * работу из {@code enqueueWork}, а {@code PlayerLoggedInEvent} наступает уже
//$$      * после того, как сервер прочитал права игрока.
//$$      */
//$$     @SubscribeEvent
//$$     public void onNegotiation(PlayerNegotiationEvent event) {
//$$         UUID uuid = ProfileId.of(event.getProfile());
//$$         event.enqueueWork(() -> runtime.onPlayerNegotiation(uuid));
//$$     }
//$$
//$$     @SubscribeEvent
//$$     public void onPlayerLoggedIn(PlayerEvent.PlayerLoggedInEvent event) {
//$$         if (event.getEntity() instanceof ServerPlayer player) {
//$$             runtime.onPlayerJoin(player);
//$$         }
//$$     }
//$$
//$$     @SubscribeEvent
//$$     public void onPlayerLoggedOut(PlayerEvent.PlayerLoggedOutEvent event) {
//$$         runtime.onPlayerLeave(event.getEntity().getUUID());
//$$     }
//$$
//$$     @SubscribeEvent
//$$     public void onRegisterCommands(RegisterCommandsEvent event) {
//$$         runtime.registerCommands(event.getDispatcher());
//$$     }
//$$
//$$     /** Мут: сообщение отменяется до того, как его увидит чей-либо чат-мод. */
//$$     @SubscribeEvent
//$$     public void onChat(ServerChatEvent event) {
//$$         if (runtime.silenced(event.getPlayer())) {
//$$             event.setCanceled(true);
//$$         }
//$$     }
//$$
//$$     /**
//$$      * Мут, обойдённый командой, не мут: с {@code /me} и {@code /msg}
//$$      * наказанный продолжает разговаривать.
//$$      */
//$$     @SubscribeEvent
//$$     public void onCommand(CommandEvent event) {
//$$         var source = event.getParseResults().getContext().getSource();
//$$         if (!(source.getEntity() instanceof ServerPlayer player)) {
//$$             return;
//$$         }
//$$         if (ChatCommands.speaks(event.getParseResults().getReader().getString())
//$$                 && runtime.silenced(player)) {
//$$             event.setCanceled(true);
//$$         }
//$$     }
//$$ }
//#endif
