//#if FORGE
//$$ package dev.noro.agent.mod;
//$$
//$$ import dev.noro.agent.core.ChatCommands;
//$$ import java.util.UUID;
//$$ import net.minecraft.server.level.ServerPlayer;
//$$ import net.minecraftforge.common.MinecraftForge;
//$$ import net.minecraftforge.event.CommandEvent;
//$$ import net.minecraftforge.event.RegisterCommandsEvent;
//$$ import net.minecraftforge.event.ServerChatEvent;
//$$ import net.minecraftforge.event.entity.player.PlayerEvent;
//$$ import net.minecraftforge.event.entity.player.PlayerNegotiationEvent;
//$$ import net.minecraftforge.event.server.ServerStartedEvent;
//$$ import net.minecraftforge.event.server.ServerStoppingEvent;
//$$ import net.minecraftforge.eventbus.api.SubscribeEvent;
//$$ import net.minecraftforge.fml.common.Mod;
//$$ import net.minecraftforge.fml.loading.FMLPaths;
//$$ import net.minecraftforge.server.permission.events.PermissionGatherEvent;
//$$
//$$ /**
//$$  * Точка входа Forge — для 1.18.2–1.20.1, где NeoForge ещё не существовало.
//$$  * Весь файл под {@code //#if FORGE}: на остальных сборках он закомментирован.
//$$  *
//$$  * <p>От NeoForge отличается только пакетами и шиной событий, поэтому вся логика
//$$  * по-прежнему в {@link AgentRuntime}, общем для всех трёх лоадеров.
//$$  */
//$$ @Mod("noro_agent")
//$$ public final class ForgeEntry {
//$$
//$$     private final AgentRuntime runtime = new AgentRuntime();
//$$
//$$     public ForgeEntry() {
//$$         if (!runtime.init(FMLPaths.CONFIGDIR.get())) {
//$$             return;
//$$         }
//$$         MinecraftForge.EVENT_BUS.register(this);
//$$     }
//$$
//$$     @SubscribeEvent
//$$     public void onServerStarted(ServerStartedEvent event) {
//$$         runtime.onServerStarted(event.getServer());
//$$         NoroPermissionHandler.warnIfInactive(AgentRuntime.LOG);
//$$     }
//$$
//$$     /** Счётчик тиков: на Forge событие одно, фазу приносит поле. */
//$$     @SubscribeEvent
//$$     public void onTick(net.minecraftforge.event.TickEvent.ServerTickEvent event) {
//$$         if (event.phase == net.minecraftforge.event.TickEvent.Phase.START) {
//$$             runtime.meter().onTickStart();
//$$         } else {
//$$             runtime.meter().onTickEnd();
//$$         }
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
//$$     // getEntity(), а не getPlayer(): первый есть и в 1.18.2 (он объявлен ещё в
//$$     // EntityEvent), а getPlayer() исчез в 1.19. Проверка instanceof всё равно
//$$     // сужает тип, так что разницы в коде нет и директива не нужна.
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
//$$     /**
//$$      * Мут и буфер разговора: см. пояснение в NeoForgeEntry — без записи в
//$$      * кольцо срез чата на этой платформе всегда пустой.
//$$      */
//$$     @SubscribeEvent
//$$     public void onChat(ServerChatEvent event) {
//$$         // До 1.19.1 сообщение приходило строкой, дальше — компонентом.
//#if MC>=11901
//$$         String text = event.getMessage().getString();
//#else
//$$         String text = event.getMessage();
//#endif
//$$         if (runtime.checkChatMessage(event.getPlayer(), text)) {
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
