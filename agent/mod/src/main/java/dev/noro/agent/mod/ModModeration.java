package dev.noro.agent.mod;

import com.mojang.brigadier.CommandDispatcher;
import dev.noro.agent.core.AgentEvents;
import dev.noro.agent.core.AgentLink;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.MasterHttp;
import dev.noro.agent.core.Moderation;
import dev.noro.agent.core.ModerationClient;
import dev.noro.agent.core.ModerationCommands;
import dev.noro.agent.core.PlayerProfile;
import dev.noro.agent.core.RuleCatalog;
import java.util.UUID;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import org.slf4j.Logger;

/**
 * Модерация на стороне мода: команды, мут и живой канал с мастером.
 *
 * <p>Отдельно от {@link AgentRuntime}, потому что подключается к другим
 * событиям и живёт своим циклом: канал открывается со стартом сервера и
 * закрывается с его остановкой, а не вместе с инициализацией мода.
 */
final class ModModeration implements AutoCloseable {

    private final MasterHttp http;
    private final MasterClient client;
    private final Moderation moderation;
    private final RuleCatalog rules;
    private final Logger log;

    private ModBridge bridge;
    private AgentLink link;

    ModModeration(MasterHttp http, MasterClient client, Logger log) {
        this.http = http;
        this.client = client;
        this.log = log;
        this.rules = new RuleCatalog(http);
        this.moderation = new Moderation(new ModerationClient(http), client, rules, log);
    }

    /** Сервер запустился: с этого момента есть кого кикать и кому писать. */
    void start(MinecraftServer server) {
        ModVanishManager.getInstance().setServer(server);
        bridge = new ModBridge(server);
        moderation.attach(bridge);
        rules.refresh();
        link = new AgentLink(http, moderation, log);
        link.start();
    }

    /**
     * Как сообщить мастеру о случившемся в игре.
     *
     * <p>{@code null} до старта сервера: канала ещё нет, а событий — тем более.
     */
    AgentEvents events() {
        AgentLink current = link;
        return current == null ? null : current.events();
    }

    /** Дерево команд собирается на каждом лоадере своим событием. */
    void registerCommands(CommandDispatcher<CommandSourceStack> dispatcher) {
        new ModCommands(new ModerationCommands(client, moderation, log), rules).register(dispatcher);
    }

    /** Подключить перечитывание профилей: применяет их платформа. */
    void attachRefresher(dev.noro.agent.core.ProfileRefresher refresher) {
        moderation.attachRefresher(refresher);
    }

    /** Экран непущенному игроку — по шаблонам админки, а не литералом. */
    String denialText(dev.noro.agent.core.AccessGate.Denial denial) {
        return dev.noro.agent.core.DenialScreen.text(denial, moderation.templates(), rules);
    }

    /** Вход игрока: мут в силе, непрочитанные предупреждения показаны. */
    void greet(UUID uuid, PlayerProfile profile) {
        moderation.applier().greet(uuid, profile);
    }

    void forget(UUID uuid) {
        moderation.mutes().forget(uuid);
    }

    /**
     * Сказал ли замученный то, чего ему нельзя.
     *
     * @return {@code true}, если сообщение надо отменить — отказ игроку уже
     *     показан
     */
    boolean silenced(ServerPlayer player) {
        String notice = moderation.muteNotice(player.getUUID(), player.getScoreboardName(), true);
        if (notice == null) {
            return false;
        }
        bridge.actionbar(player.getUUID(), notice);
        return true;
    }

    dev.noro.agent.core.automod.ChatFilters.Result checkChatMessage(UUID uuid, String text) {
        return moderation.checkChatMessage(uuid, text);
    }

    @Override
    public void close() {
        if (link != null) {
            link.close();
        }
        moderation.close();
    }
}
