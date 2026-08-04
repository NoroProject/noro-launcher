package dev.noro.agent.paper;

import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.HeartbeatTask;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.LuckPermsSupport;
import dev.noro.agent.core.RoleApplier;
import java.util.List;
import org.bukkit.permissions.Permission;
import org.bukkit.plugin.java.JavaPlugin;

/** Точка входа плагина: собирает core-части и вешает слушатель входа. */
public final class NoroAgentPlugin extends JavaPlugin {

    private HeartbeatTask heartbeat;

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

        MasterClient client = new MasterClient(config);
        RoleApplier roleSync = LuckPermsSupport.tryCreate(getSLF4JLogger());
        PaperPermissions permissions = new PaperPermissions(this);

        getServer()
                .getPluginManager()
                .registerEvents(
                        new LoginListener(config, client, roleSync, permissions, getSLF4JLogger()), this);
        getServer().getPluginManager().registerEvents(permissions, this);

        heartbeat = new HeartbeatTask(client, new PaperServerStatus(getServer()), config, getSLF4JLogger());
        heartbeat.start();

        // Права плагины регистрируют в своих onEnable, а первый тик наступает уже
        // после всех — только там каталог полон. Собираем его в главном потоке,
        // а отправляем мимо: сеть на тике держать нельзя. Раз за старт — дальше
        // набор не меняется.
        getServer().getScheduler().runTaskLater(this, () -> {
            List<String> nodes = getServer().getPluginManager().getPermissions().stream()
                    .map(Permission::getName)
                    .sorted()
                    .toList();
            getServer().getScheduler().runTaskAsynchronously(this, () -> reportNodes(client, nodes));
        }, 1L);
    }

    private void reportNodes(MasterClient client, List<String> nodes) {
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
    }
}
