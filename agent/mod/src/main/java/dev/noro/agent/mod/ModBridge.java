package dev.noro.agent.mod;

import dev.noro.agent.core.GameBridge;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.UUID;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;

/**
 * Модерация глазами Minecraft: кто в сети, кого отключить и что ему сказать.
 *
 * <p>Один класс на Fabric, Forge и NeoForge — все вызовы отсюда есть на всех
 * трёх. Расходятся только версии игры, и это разведено препроцессором.
 */
final class ModBridge implements GameBridge {

    private final MinecraftServer server;

    ModBridge(MinecraftServer server) {
        this.server = server;
    }

    @Override
    public Optional<UUID> onlineUuid(String name) {
        ServerPlayer player = server.getPlayerList().getPlayerByName(name);
        return Optional.ofNullable(player).map(ServerPlayer::getUUID);
    }

    @Override
    public Collection<String> onlineNames() {
        List<ServerPlayer> players = server.getPlayerList().getPlayers();
        Collection<String> names = new ArrayList<>(players.size());
        for (ServerPlayer player : players) {
            names.add(player.getScoreboardName());
        }
        return names;
    }

    @Override
    public void kick(UUID uuid, String message) {
        // Наказание приходит из потока WebSocket, а сетевую сессию игрока можно
        // трогать только из главного. MinecraftServer сам является Executor'ом.
        onMain(uuid, player -> player.connection.disconnect(text(message)));
    }

    @Override
    public void tell(UUID uuid, String message) {
        onMain(uuid, player -> send(player, text(message)));
    }

    @Override
    public void actionbar(UUID uuid, String message) {
        onMain(uuid, player -> {
            //#if MC>=260000
            //$$ player.sendSystemMessage(text(message));
            //#else
            player.displayClientMessage(text(message), true);
            //#endif
        });
    }

    @Override
    public void announce(String message) {
        server.execute(() -> {
            Component component = text(message);
            for (ServerPlayer player : server.getPlayerList().getPlayers()) {
                send(player, component);
            }
        });
    }

    @Override
    public void announceToPermission(String permission, String message) {
        server.execute(() -> {
            Component component = text(message);
            for (ServerPlayer player : server.getPlayerList().getPlayers()) {
                dev.noro.agent.core.PlayerProfile profile = dev.noro.agent.core.NoroAgentApi.profile(player.getUUID());
                if (profile != null && profile.permissions() != null && profile.permissions().contains(permission)) {
                    send(player, component);
                }
            }
        });
    }

    @Override
    public boolean teleport(UUID who, String world, double x, double y, double z) {
        ServerPlayer player = server.getPlayerList().getPlayer(who);
        if (player == null) {
            return false;
        }
        net.minecraft.server.level.ServerLevel level = levelByName(world);
        if (level == null) {
            return false;
        }
        onMain(who, target -> moveTo(target, level, x, y, z));
        return true;
    }

    @Override
    public boolean teleportTo(UUID who, UUID target) {
        ServerPlayer to = server.getPlayerList().getPlayer(target);
        if (to == null || server.getPlayerList().getPlayer(who) == null) {
            return false;
        }
        onMain(who, mover -> moveTo(mover, levelOf(to), to.getX(), to.getY(), to.getZ()));
        return true;
    }

    @Override
    public Optional<Position> position(UUID who) {
        ServerPlayer player = server.getPlayerList().getPlayer(who);
        if (player == null) {
            return Optional.empty();
        }
        return Optional.of(new Position(
                levelOf(player).dimension().location().toString(),
                player.getX(),
                player.getY(),
                player.getZ()));
    }

    /**
     * Инвентарь цели контейнером — как {@code /invsee}.
     *
     * <p>Заголовок окна — ник владельца: у модератора в разборе открыто и своё, и
     * чужое, и перепутать их — значит выбросить вещи не тому.
     */
    @Override
    public boolean openInventory(UUID viewer, UUID target) {
        ServerPlayer watched = server.getPlayerList().getPlayer(target);
        if (watched == null || server.getPlayerList().getPlayer(viewer) == null) {
            return false;
        }
        // Через конструктор, а не через `ChestMenu.fourRows`: тот собирает окно
        // без контейнера — он для клиента, которому слоты приезжают по сети.
        return open(viewer, watched.getScoreboardName(), (id, inventory, player) ->
                new net.minecraft.world.inventory.ChestMenu(
                        net.minecraft.world.inventory.MenuType.GENERIC_9x4,
                        id,
                        inventory,
                        new PeekContainer(watched.getInventory()),
                        4));
    }

    @Override
    public boolean openEnderChest(UUID viewer, UUID target) {
        ServerPlayer watched = server.getPlayerList().getPlayer(target);
        if (watched == null || server.getPlayerList().getPlayer(viewer) == null) {
            return false;
        }
        return open(viewer, watched.getScoreboardName(), (id, inventory, player) ->
                net.minecraft.world.inventory.ChestMenu.threeRows(
                        id, inventory, watched.getEnderChestInventory()));
    }

    private boolean open(UUID viewer, String title, net.minecraft.world.inventory.MenuConstructor menu) {
        onMain(viewer, player -> player.openMenu(
                new net.minecraft.world.SimpleMenuProvider(menu, text(title))));
        return true;
    }

    /**
     * Пак с плашками — выдачей сервера, а не файлом в папке.
     *
     * <p>Необязательный: отказ не выкидывает игрока, он просто останется с
     * текстовым префиксом. Требовать пак ради оформления чата — плохая сделка.
     */
    @Override
    public boolean sendResourcePack(UUID who, String url, String sha1) {
        if (server.getPlayerList().getPlayer(who) == null) {
            return false;
        }
        // Пакет сменился в 1.20.3: до неё выдача была одиночной, после — стопкой
        // паков, каждый со своим номером.
        //#if MC>=12005
        onMain(who, player -> player.connection.send(
                new net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket(
                        PACK_ID, url, sha1, false, java.util.Optional.empty())));
        //#elseif MC>=12003
        //$$ // В 1.20.3 и 1.20.4 подсказка передавалась компонентом, а не Optional.
        //$$ onMain(who, player -> player.connection.send(
        //$$         new net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket(
        //$$                 PACK_ID, url, sha1, false, null)));
        //#else
        //$$ // До 1.20.3 клиент применял пак только при перезапуске, да и пакет
        //$$ // за две версии переезжал между пакетами. Честнее сказать «не умею»:
        //$$ // на этих версиях пак доедет через лаунчер или server.properties.
        //$$ return false;
        //#endif
        //#if MC>=12003
        return true;
        //#endif
    }

    /** Один и тот же номер пака: выдача с ним же заменяет прошлую, а не копит. */
    private static final UUID PACK_ID = UUID.fromString("9070e15f-0000-4000-8000-000000000001");

    /**
     * Наблюдение: зритель уходит в spectator и цепляется камерой за цель.
     * Возврат в выживание — {@link #stopSpectate}, и делать это обязан режим
     * разбора: сам по себе игрок из чужой камеры не выберется.
     */
    @Override
    public boolean spectate(UUID viewer, UUID target) {
        ServerPlayer watcher = server.getPlayerList().getPlayer(viewer);
        ServerPlayer watched = server.getPlayerList().getPlayer(target);
        if (watcher == null || watched == null) {
            return false;
        }
        onMain(viewer, player -> {
            player.setGameMode(net.minecraft.world.level.GameType.SPECTATOR);
            player.setCamera(watched);
        });
        return true;
    }

    @Override
    public boolean stopSpectate(UUID viewer) {
        ServerPlayer watcher = server.getPlayerList().getPlayer(viewer);
        if (watcher == null) {
            return false;
        }
        onMain(viewer, player -> {
            player.setCamera(player);
            player.setGameMode(net.minecraft.world.level.GameType.SURVIVAL);
        });
        return true;
    }

    /** Снимок инвентаря строками «предмет xN» — доказательство в дело. */
    @Override
    public List<Slot> inventory(UUID who) {
        ServerPlayer player = server.getPlayerList().getPlayer(who);
        if (player == null) {
            return List.of();
        }
        // Через Container, а не через поле `items`: в новых версиях оно
        // приватное, а размер и слоты читаются одинаково во всём диапазоне.
        net.minecraft.world.entity.player.Inventory inventory = player.getInventory();
        List<Slot> out = new ArrayList<>();
        for (int slot = 0; slot < inventory.getContainerSize(); slot++) {
            net.minecraft.world.item.ItemStack stack = inventory.getItem(slot);
            if (!stack.isEmpty()) {
                out.add(new Slot(
                        slot, itemId(stack), stack.getCount(),
                        stack.getHoverName().getString(), encode(stack)));
            }
        }
        return out;
    }

    /** {@code minecraft:stone}. Реестры переехали из {@code Registry} в 1.19.3. */
    private static String itemId(net.minecraft.world.item.ItemStack stack) {
        //#if MC>=11903
        return net.minecraft.core.registries.BuiltInRegistries.ITEM
                .getKey(stack.getItem())
                .toString();
        //#else
        //$$ return net.minecraft.core.Registry.ITEM.getKey(stack.getItem()).toString();
        //#endif
    }

    /**
     * Предмет целиком, строкой.
     *
     * <p>С 1.20.5 предмет описан компонентами, и его кодек тянет за собой
     * реестры — зачарования и эффекты записаны в них ссылками. Поэтому контекст
     * берётся из мира, а не пустой. До 1.20.5 компонентов не было: там предмет
     * это NBT, и {@code save} отдаёт его целиком без всяких реестров.
     */
    private String encode(net.minecraft.world.item.ItemStack stack) {
        //#if MC>=12005
        try {
            var ops = server.registryAccess()
                    .createSerializationContext(com.mojang.serialization.JsonOps.INSTANCE);
            return net.minecraft.world.item.ItemStack.CODEC
                    .encodeStart(ops, stack)
                    .result()
                    .map(Object::toString)
                    .orElse(null);
        } catch (Exception e) {
            return null;
        }
        //#else
        //$$ return stack.save(new net.minecraft.nbt.CompoundTag()).toString();
        //#endif
    }

    /** Мир игрока: {@code serverLevel()} появился только в 1.20. */
    private static net.minecraft.server.level.ServerLevel levelOf(ServerPlayer player) {
        //#if MC>=12000
        return player.serverLevel();
        //#else
        //$$ return player.getLevel();
        //#endif
    }

    private net.minecraft.server.level.ServerLevel levelByName(String world) {
        if (world == null) {
            return null;
        }
        for (net.minecraft.server.level.ServerLevel level : server.getAllLevels()) {
            if (level.dimension().location().toString().equals(world)) {
                return level;
            }
        }
        return null;
    }

    /**
     * Перемещение между версиями меняло сигнатуру не раз, поэтому здесь одна
     * точка: остальной код зовёт её и не знает про препроцессор.
     */
    private static void moveTo(
            ServerPlayer player, net.minecraft.server.level.ServerLevel level, double x, double y, double z) {
        //#if MC>=12102
        //$$ player.teleportTo(level, x, y, z, java.util.Set.of(), player.getYRot(), player.getXRot(), false);
        //#else
        player.teleportTo(level, x, y, z, player.getYRot(), player.getXRot());
        //#endif
    }

    private void onMain(UUID uuid, java.util.function.Consumer<ServerPlayer> action) {
        server.execute(() -> {
            ServerPlayer player = server.getPlayerList().getPlayer(uuid);
            // Игрок мог выйти сам, пока кадр летел: это не ошибка.
            if (player != null) {
                action.accept(player);
            }
        });
    }

    /**
     * Системное сообщение игроку.
     *
     * <p>До 1.19 у сообщений был отправитель, и системные слал «никто» —
     * {@code Util.NIL_UUID}. С приходом подписанного чата параметр исчез.
     */
    private static void send(ServerPlayer player, Component component) {
        //#if MC>=11900
        player.sendSystemMessage(component);
        //#else
        //$$ player.sendMessage(component, net.minecraft.Util.NIL_UUID);
        //#endif
    }

    /**
     * Строка с {@code §}-кодами. Ванильный рендер разбирает их сам, поэтому
     * разбирать цвета в стили здесь незачем — и это единственный формат,
     * одинаковый на всём диапазоне версий.
     */
    private static Component text(String message) {
        return ModText.parse(message);
    }
}
