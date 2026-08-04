//#if FORGE || NEOFORGE
package dev.noro.agent.mod;

//#if FORGE
import net.minecraftforge.server.permission.PermissionAPI;
import net.minecraftforge.server.permission.handler.IPermissionHandler;
import net.minecraftforge.server.permission.nodes.PermissionDynamicContext;
import net.minecraftforge.server.permission.nodes.PermissionNode;
import net.minecraftforge.server.permission.nodes.PermissionTypes;
//#else
//$$ import net.neoforged.neoforge.server.permission.PermissionAPI;
//$$ import net.neoforged.neoforge.server.permission.handler.IPermissionHandler;
//$$ import net.neoforged.neoforge.server.permission.nodes.PermissionDynamicContext;
//$$ import net.neoforged.neoforge.server.permission.nodes.PermissionNode;
//$$ import net.neoforged.neoforge.server.permission.nodes.PermissionTypes;
//#endif
//#if MC>=12111
//$$ import net.minecraft.resources.Identifier;
//#else
import net.minecraft.resources.ResourceLocation;
//#endif
import java.util.Collection;
import java.util.List;
import java.util.Set;
import java.util.UUID;
import net.minecraft.server.level.ServerPlayer;
import org.slf4j.Logger;

/**
 * Права мастера как обработчик прав лоадера.
 *
 * <p>Файл целиком под {@code //#if FORGE || NEOFORGE}: на Fabric этого API нет.
 * Между Forge и NeoForge расходятся только пакеты, поэтому директива закрывает
 * импорты, а тело класса общее.
 *
 * <p>{@code tryParse} вместо конструктора и {@code parse}: конструктор стал
 * приватным в 1.21, а {@code parse} до неё не существовало. {@code tryParse}
 * есть на всём диапазоне и на константе не промахнётся.
 */
final class NoroPermissionHandler implements IPermissionHandler {

    // В 1.21.11 ResourceLocation переименован в Identifier. Объявление и
    // геттер держим рядом, чтобы разрыв закрывался одной директивой.
//#if MC>=12111
//$$     static final Identifier IDENTIFIER = Identifier.tryParse("noro:agent");
//$$
//$$     @Override
//$$     public Identifier getIdentifier() {
//$$         return IDENTIFIER;
//$$     }
//#else
    static final ResourceLocation IDENTIFIER = ResourceLocation.tryParse("noro:agent");

    @Override
    public ResourceLocation getIdentifier() {
        return IDENTIFIER;
    }
//#endif

    private final Set<PermissionNode<?>> nodes;
    private final ModPermissions permissions;

    NoroPermissionHandler(Collection<PermissionNode<?>> nodes, ModPermissions permissions) {
        this.nodes = Set.copyOf(nodes);
        this.permissions = permissions;
    }

    @Override
    public Set<PermissionNode<?>> getRegisteredNodes() {
        return nodes;
    }

    @Override
    public <T> T getPermission(ServerPlayer player, PermissionNode<T> node, PermissionDynamicContext<?>... context) {
        return resolve(player, player.getUUID(), node, context);
    }

    @Override
    public <T> T getOfflinePermission(UUID uuid, PermissionNode<T> node, PermissionDynamicContext<?>... context) {
        return resolve(null, uuid, node, context);
    }

    /**
     * Мастер знает только про булевы права: «выдано» или «нет». Узлы других
     * типов — число, строка, компонент — он не описывает, и подменять их нечем.
     *
     * <p>Отсутствие права у мастера — это «ему нечего сказать», а не запрет,
     * поэтому дальше спрашивается дефолт узла. Иначе оператор и консоль
     * потеряли бы всё, что им даёт уровень прав: его мастер не выдаёт.
     */
    private <T> T resolve(ServerPlayer player, UUID uuid, PermissionNode<T> node, PermissionDynamicContext<?>... context) {
        if (PermissionTypes.BOOLEAN.equals(node.getType()) && permissions.of(uuid).has(node.getNodeName())) {
            return node.getType().typeToken().cast(Boolean.TRUE);
        }
        return node.getDefaultResolver().resolve(player, uuid, context);
    }

    /** Имена узлов для каталога на мастере. Порядок стабильный — ради diff'а в админке. */
    static List<String> names(Collection<PermissionNode<?>> nodes) {
        return nodes.stream().map(PermissionNode::getNodeName).distinct().sorted().toList();
    }

    /**
     * Зарегистрироваться мало: лоадер выбирает обработчик по значению
     * {@code permissionHandler} в своём server-конфиге, а по умолчанию там стоит
     * встроенный. Без этой строки в логе админ ищет причину вслепую.
     */
    static void warnIfInactive(Logger log) {
        var active = PermissionAPI.getActivePermissionHandler();
        if (IDENTIFIER.equals(active)) {
            log.info("Permission handler {} is in charge, master permissions are live", IDENTIFIER);
            return;
        }
        log.warn("Permission handler in charge is {}, so master permissions are ignored."
                + " Set permissionHandler = \"{}\" in config/neoforge-server.toml"
                + " (config/forge-server.toml on Forge) and restart.", active, IDENTIFIER);
    }
}
//#endif
