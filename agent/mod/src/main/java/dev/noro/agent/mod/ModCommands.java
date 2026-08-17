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
    private final GameBridge bridge;
    private final RuleCatalog rules;

    ModCommands(ModerationCommands commands, GameBridge bridge, RuleCatalog rules) {
        this.commands = commands;
        this.bridge = bridge;
        this.rules = rules;
    }

    void register(CommandDispatcher<CommandSourceStack> dispatcher) {
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
        return Commands.literal(name)
                .requires(source -> allowed(source, ModerationCommands.permission(kind)))
                .then(Commands.argument("player", StringArgumentType.word())
                        .suggests((context, builder) -> SharedSuggestionProvider.suggest(bridge.onlineNames(), builder))
                        .executes(context -> {
                            commands.punish(
                                    new ModSender(context.getSource()),
                                    kind,
                                    new String[] { StringArgumentType.getString(context, "player") });
                            return 1;
                        })
                        .then(Commands.argument("args", StringArgumentType.greedyString())
                                .suggests(this::suggestRules)
                                .executes(context -> {
                                    String rest = StringArgumentType.getString(context, "args");
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
                        .suggests((context, builder) -> SharedSuggestionProvider.suggest(bridge.onlineNames(), builder))
                        .executes(context -> {
                            action.accept(context.getSource(), StringArgumentType.getString(context, "player"));
                            return 1;
                        }));
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
     * Подсказка кодов правил: {@code @chat.spam}. Срок Brigadier подсказать не
     * даст — он внутри жадной строки, и позиции слова там уже нет.
     */
    private java.util.concurrent.CompletableFuture<com.mojang.brigadier.suggestion.Suggestions> suggestRules(
            CommandContext<CommandSourceStack> context, com.mojang.brigadier.suggestion.SuggestionsBuilder builder) {
        String typed = builder.getRemaining();
        int space = typed.lastIndexOf(' ');
        String word = space < 0 ? typed : typed.substring(space + 1);
        if (!word.startsWith("@")) {
            return builder.buildFuture();
        }
        return SharedSuggestionProvider.suggest(rules.matching(word.substring(1)), builder);
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
