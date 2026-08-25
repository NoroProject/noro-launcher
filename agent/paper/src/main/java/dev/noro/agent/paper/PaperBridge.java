package dev.noro.agent.paper;

import dev.noro.agent.core.GameBridge;
import java.util.Collection;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.Collectors;
import net.kyori.adventure.text.Component;
import org.bukkit.Server;
import org.bukkit.entity.Player;
import org.bukkit.plugin.Plugin;

/** Модерация глазами Bukkit: кто в сети, кого отключить и что ему сказать. */
final class PaperBridge implements GameBridge {

    private final Plugin plugin;
    private final Server server;

    PaperBridge(Plugin plugin) {
        this.plugin = plugin;
        this.server = plugin.getServer();
    }

    @Override
    public Optional<UUID> onlineUuid(String name) {
        Player player = server.getPlayerExact(name);
        return Optional.ofNullable(player).map(Player::getUniqueId);
    }

    @Override
    public Collection<String> onlineNames() {
        return server.getOnlinePlayers().stream().map(Player::getName).collect(Collectors.toList());
    }

    @Override
    public boolean teleport(UUID who, String world, double x, double y, double z) {
        Player player = server.getPlayer(who);
        org.bukkit.World target = world == null ? null : server.getWorld(shortName(world));
        if (player == null || target == null) {
            return false;
        }
        onMain(who, mover -> mover.teleport(new org.bukkit.Location(target, x, y, z)));
        return true;
    }

    @Override
    public boolean teleportTo(UUID who, UUID target) {
        Player to = server.getPlayer(target);
        if (to == null || server.getPlayer(who) == null) {
            return false;
        }
        onMain(who, mover -> mover.teleport(to.getLocation()));
        return true;
    }

    @Override
    public Optional<Position> position(UUID who) {
        Player player = server.getPlayer(who);
        if (player == null) {
            return Optional.empty();
        }
        org.bukkit.Location at = player.getLocation();
        return Optional.of(new Position(at.getWorld().getName(), at.getX(), at.getY(), at.getZ()));
    }

    /**
     * Свечение цели — только для одного зрителя.
     *
     * <p>Bukkit такого не умеет: {@code setGlowing} светит всем, и нарушитель
     * узнал бы о разборе раньше модератора. Пока не появится пакетная подмена
     * (та же точка, что у ваниша), честнее ответить «нет».
     */
    @Override
    public boolean glow(UUID viewer, UUID target, boolean on) {
        return false;
    }

    @Override
    public boolean spectate(UUID viewer, UUID target) {
        Player watcher = server.getPlayer(viewer);
        Player watched = server.getPlayer(target);
        if (watcher == null || watched == null) {
            return false;
        }
        onMain(viewer, player -> {
            player.setGameMode(org.bukkit.GameMode.SPECTATOR);
            player.setSpectatorTarget(watched);
        });
        return true;
    }

    @Override
    public boolean stopSpectate(UUID viewer) {
        Player watcher = server.getPlayer(viewer);
        if (watcher == null) {
            return false;
        }
        onMain(viewer, player -> {
            player.setSpectatorTarget(null);
            player.setGameMode(org.bukkit.GameMode.SURVIVAL);
        });
        return true;
    }

    /**
     * Инвентарь цели контейнером: Bukkit умеет это напрямую.
     *
     * <p>Открывается живой инвентарь, а не копия, — изъятое действительно
     * пропадает у игрока. Правки видны ему сразу, если он в сети.
     */
    @Override
    public boolean openInventory(UUID viewer, UUID target) {
        Player watcher = server.getPlayer(viewer);
        Player watched = server.getPlayer(target);
        if (watcher == null || watched == null) {
            return false;
        }
        onMain(viewer, player -> player.openInventory(watched.getInventory()));
        return true;
    }

    @Override
    public boolean openEnderChest(UUID viewer, UUID target) {
        Player watcher = server.getPlayer(viewer);
        Player watched = server.getPlayer(target);
        if (watcher == null || watched == null) {
            return false;
        }
        onMain(viewer, player -> player.openInventory(watched.getEnderChest()));
        return true;
    }

    @Override
    public java.util.List<Slot> inventory(UUID who) {
        Player player = server.getPlayer(who);
        if (player == null) {
            return java.util.List.of();
        }
        java.util.List<Slot> out = new java.util.ArrayList<>();
        org.bukkit.inventory.ItemStack[] contents = player.getInventory().getContents();
        for (int slot = 0; slot < contents.length; slot++) {
            org.bukkit.inventory.ItemStack stack = contents[slot];
            if (stack == null || stack.getType().isAir()) {
                continue;
            }
            // `getKey()` даёт `minecraft:diamond_pickaxe` — тот же вид, что у
            // мода, чтобы панель рисовала иконку одинаково на обеих платформах.
            // NBT здесь нет: у Bukkit нет доступа к ванильному кодеку предмета,
            // а собирать его руками значило бы держать вторую, расходящуюся
            // версию формата. Панель в этом случае покажет снимок текстом.
            out.add(new Slot(
                    slot,
                    stack.getType().getKey().toString(),
                    stack.getAmount(),
                    stack.getType().name().toLowerCase(java.util.Locale.ROOT),
                    null));
        }
        return out;
    }

    /** `minecraft:overworld` — имя измерения мода; у Bukkit миры зовутся иначе. */
    private static String shortName(String world) {
        int colon = world.indexOf(':');
        return colon < 0 ? world : world.substring(colon + 1);
    }

    @Override
    public void kick(UUID uuid, String message) {
        // Отключение обязано идти в главном потоке: наказание приходит из
        // потока WebSocket, а трогать сетевую сессию оттуда нельзя.
        onMain(uuid, player -> player.kick(text(message)));
    }

    @Override
    public void tell(UUID uuid, String message) {
        onMain(uuid, player -> player.sendMessage(text(message)));
    }

    @Override
    public void actionbar(UUID uuid, String message) {
        onMain(uuid, player -> player.sendActionBar(text(message)));
    }

    @Override
    public void announce(String message) {
        server.getScheduler().runTask(plugin, () -> server.broadcast(text(message)));
    }

    @Override
    public void announceToPermission(String permission, String message) {
        server.getScheduler().runTask(plugin, () -> {
            Component msg = text(message);
            for (Player p : server.getOnlinePlayers()) {
                if (p.hasPermission(permission)) {
                    p.sendMessage(msg);
                }
            }
        });
    }

    private void onMain(UUID uuid, java.util.function.Consumer<Player> action) {
        server.getScheduler().runTask(plugin, () -> {
            Player player = server.getPlayer(uuid);
            // Игрок мог выйти сам, пока кадр летел: это не ошибка.
            if (player != null) {
                action.accept(player);
            }
        });
    }

    /** Разбор разметки общий с модами: см. {@link PaperText}. */
    private static Component text(String message) {
        return PaperText.parse(message);
    }
}
