package dev.noro.agent.mod;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.context.CommandContext;
import dev.noro.agent.core.GameBridge;
import dev.noro.agent.core.ModerationCommands;
import dev.noro.agent.core.RuleCatalog;
import java.util.function.BiConsumer;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.commands.SharedSuggestionProvider;

/**
 * Команды модерации в Brigadier. Разбор и логика — в core, здесь только дерево
 * команд и подсказки.
 *
 * <p>Аргументы после ника берутся одной жадной строкой, а не отдельными узлами:
 * срок и код правила необязательны, и в Brigadier это дало бы восемь веток на
 * команду вместо одного разбора, общего с Paper.
 */
final class ModCommands {

    private final ModerationCommands commands;
    private final RuleCatalog rules;
    private final ModSuggestions suggestions;

    ModCommands(ModerationCommands commands, RuleCatalog rules) {
        this.commands = commands;
        this.rules = rules;
        this.suggestions = new ModSuggestions(commands, rules);
    }

    void register(CommandDispatcher<CommandSourceStack> dispatcher) {
        AgentRuntime.LOG.info("Registering moderation commands in Brigadier: ban, serverban, mute, warn, unban, unmute, history");
        dispatcher.register(punish("ban", "ban"));
        dispatcher.register(punish("serverban", "server_ban"));
        dispatcher.register(punish("mute", "mute"));
        dispatcher.register(punish("warn", "warn"));
        dispatcher.register(target("unban", "noro.mod.punish.revoke", (source, name) ->
                commands.revoke(new ModSender(source), "ban", new String[] {name})));
        dispatcher.register(target("unmute", "noro.mod.punish.revoke", (source, name) ->
                commands.revoke(new ModSender(source), "mute", new String[] {name})));
        dispatcher.register(target("history", "noro.mod.punish.view", (source, name) ->
                commands.history(new ModSender(source), new String[] {name})));
    }

    private LiteralArgumentBuilder<CommandSourceStack> punish(String name, String kind) {
        boolean timed = ModerationCommands.TIMED.contains(kind);
        return Commands.literal(name)
                .requires(source -> allowed(source, ModerationCommands.permission(kind)))
                .then(Commands.argument("player", StringArgumentType.word())
                        .suggests((context, builder) -> SharedSuggestionProvider.suggest(onlineNames(context.getSource()), builder))
                        .executes(context -> {
                            commands.punish(
                                    new ModSender(context.getSource()),
                                    kind,
                                    new String[] { StringArgumentType.getString(context, "player") });
                            return 1;
                        })
                        .then(Commands.argument("reason", StringArgumentType.greedyString())
                                .suggests((context, builder) -> suggestions.suggest(context, builder, timed))
                                .executes(context -> {
                                    String rest = StringArgumentType.getString(context, "reason");
                                    commands.punish(
                                            new ModSender(context.getSource()),
                                            kind,
                                            arguments(context, rest));
                                    return 1;
                                })));
    }

    private LiteralArgumentBuilder<CommandSourceStack> target(
            String name, String permission, BiConsumer<CommandSourceStack, String> action) {
        return Commands.literal(name)
                .requires(source -> allowed(source, permission))
                .then(Commands.argument("player", StringArgumentType.word())
                        .suggests((context, builder) -> SharedSuggestionProvider.suggest(onlineNames(context.getSource()), builder))
                        .executes(context -> {
                            action.accept(context.getSource(), StringArgumentType.getString(context, "player"));
                            return 1;
                        }));
    }

    private static java.util.Collection<String> onlineNames(CommandSourceStack source) {
        if (source == null || source.getServer() == null) {
            return java.util.List.of();
        }
        var players = source.getServer().getPlayerList().getPlayers();
        java.util.List<String> names = new java.util.ArrayList<>(players.size());
        for (var player : players) {
            names.add(player.getScoreboardName());
        }
        return names;
    }

    /** Ник плюс остаток строки словами — ровно то, что ждёт разбор в core. */
    private static String[] arguments(CommandContext<CommandSourceStack> context, String rest) {
        String player = StringArgumentType.getString(context, "player");
        String[] tail = rest.strip().split("\\s+");
        String[] args = new String[tail.length + 1];
        args[0] = player;
        System.arraycopy(tail, 0, args, 1, tail.length);
        return args;
    }

    /**
     * Консоли команда видна всегда: за ней сервер, а не человек, и прав мастера
     * у неё нет по определению.
     */
    private static boolean allowed(CommandSourceStack source, String permission) {
        ModSender sender = new ModSender(source);
        return sender.console() || sender.has(permission);
    }
}
