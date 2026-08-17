package dev.noro.agent.paper;

import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.AgentLink;
import dev.noro.agent.core.HeartbeatTask;
import dev.noro.agent.core.LuckPermsSupport;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.MasterHttp;
import dev.noro.agent.core.Moderation;
import dev.noro.agent.core.ModerationClient;
import dev.noro.agent.core.ModerationCommands;
import dev.noro.agent.core.NoroAgentApi;
import dev.noro.agent.core.ProfileCache;
import dev.noro.agent.core.RoleApplier;
import dev.noro.agent.core.RuleCatalog;
import org.bukkit.command.PluginCommand;
import org.bukkit.permissions.Permission;
import org.bukkit.plugin.java.JavaPlugin;

/** Точка входа плагина: собирает core-части и вешает слушатели. */
public final class NoroAgentPlugin extends JavaPlugin {

    private HeartbeatTask heartbeat;
    private AgentLink link;

    @Override
    public void onEnable() {
        AgentConfig config;
        try {
            config = AgentConfig.load(getDataFolder().toPath().resolve("noro-agent.properties"));
        } catch (RuntimeException e) {
            // Без секрета агент бесполезен, а молча работающий вхолостую плагин
            // хуже выключенного: сервер будет числиться offline без объяснений.
            getSLF4JLogger().error("Cannot start: {}", e.getMessage());
            getServer().getPluginManager().disablePlugin(this);
            return;
        }
        getSLF4JLogger().info("Starting against {}", config);

        MasterHttp http = new MasterHttp(config);
        MasterClient client = new MasterClient(http);
        RoleApplier roleSync = LuckPermsSupport.tryCreate(getSLF4JLogger());
        PaperPermissions permissions = new PaperPermissions(this);
        // Общий с NoroAgentApi: чужие плагины читают профиль оттуда же.
        ProfileCache profiles = NoroAgentApi.cache();
        PaperBridge bridge = new PaperBridge(this);
        Moderation moderation = new Moderation(new ModerationClient(http), getSLF4JLogger());
        moderation.attach(bridge);

        getServer()
                .getPluginManager()
                .registerEvents(
                        new LoginListener(
                                config, client, roleSync, permissions, profiles, moderation, getSLF4JLogger()),
                        this);
        getServer().getPluginManager().registerEvents(permissions, this);
        getServer().getPluginManager().registerEvents(new ChatGuard(moderation), this);
        PlaceholderSupport.register(this, profiles, getSLF4JLogger());
        registerCommands(client, moderation, http, bridge);

        heartbeat = new HeartbeatTask(client, new PaperServerStatus(getServer()), config, getSLF4JLogger());
        heartbeat.start();
        // Живой канал: бан из панели должен выкинуть игрока сейчас, а не к его
        // следующему входу.
        link = new AgentLink(http, moderation, getSLF4JLogger());
        link.start();

        // Права плагины регистрируют в своих onEnable, а первый тик наступает уже
        // после всех — только там каталог полон. Собираем его в главном потоке,
        // а отправляем мимо: сеть на тике держать нельзя. Раз за старт — дальше
        // набор не меняется.
        // Отправку подключаем сразу: чужой плагин мог заявить свои узлы уже в
        // своём onEnable, до нашего тика, и они ждут в каталоге.
        NoroAgentApi.permissionNodes().attach(
                nodes -> getServer().getScheduler().runTaskAsynchronously(this, () -> reportNodes(client, nodes)));
        getServer().getScheduler().runTaskLater(this, () -> NoroAgentApi.permissionNodes().register(
                getServer().getPluginManager().getPermissions().stream()
                        .map(Permission::getName)
                        .toList()), 1L);
    }

    private void registerCommands(
            MasterClient client, Moderation moderation, MasterHttp http, PaperBridge bridge) {
        RuleCatalog rules = new RuleCatalog(http);
        rules.refresh();
        ModerationCommand handler = new ModerationCommand(
                new ModerationCommands(client, moderation, getSLF4JLogger()), bridge, rules);
        for (String name : ModerationCommand.names()) {
            PluginCommand command = getCommand(name);
            // Команда могла быть занята другим плагином — тогда её просто нет
            // у нас, и подменять чужой обработчик мы не станем.
            if (command == null) {
                getSLF4JLogger().warn("Command /{} is taken by another plugin, skipping it", name);
                continue;
            }
            command.setExecutor(handler);
            command.setTabCompleter(handler);
        }
    }

    private void reportNodes(MasterClient client, java.util.Collection<String> nodes) {
        try {
            client.reportNodes(nodes);
            getSLF4JLogger().info("Reported {} permission nodes to master", nodes.size());
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        } catch (Exception e) {
            // Каталог — подсказка для админки, а не условие работы сервера.
            getSLF4JLogger().warn("Cannot report permission nodes: {}", e.getMessage());
        }
    }

    @Override
    public void onDisable() {
        if (heartbeat != null) {
            heartbeat.close();
        }
        if (link != null) {
            link.close();
        }
    }
}
