package dev.noro.agent.mod;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.context.CommandContext;
import dev.noro.agent.core.CheckCommand;
import dev.noro.agent.core.FreezeCommand;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.Moderation;
import dev.noro.agent.core.ModerationCommands;
import dev.noro.agent.core.ReportCommand;
import dev.noro.agent.core.RuleCatalog;
import dev.noro.agent.core.RuleCommands;
import java.util.function.BiConsumer;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.commands.SharedSuggestionProvider;
import org.slf4j.Logger;

/**
 * Команды модерации в Brigadier. Разбор и логика — в core, здесь только дерево
 * команд и подсказки.
 */
final class ModCommands {

    private final ModerationCommands commands;
    private final FreezeCommand freezeCmd;
    private final ReportCommand reportCmd;
    private final CheckCommand checkCmd;
    private final RuleCommands ruleCmds;
    private final RuleCatalog rules;
    private final ModSuggestions suggestions;
    private final Moderation moderation;

    ModCommands(MasterClient master, Moderation moderation, RuleCatalog rules, Logger log) {
        this.moderation = moderation;
        this.commands = new ModerationCommands(master, moderation, log);
        this.freezeCmd = new FreezeCommand(master, moderation, log);
        // Мост спрашиваем лениво: applier появляется со стартом канала, а дерево
        // команд строится раньше — как и у CaseCommands ниже.
        this.reportCmd = new ReportCommand(master, () -> {
            var applier = moderation.applier();
            return applier == null ? null : applier.bridge();
        }, log);
        this.checkCmd = new CheckCommand(master, log);
        this.ruleCmds = new RuleCommands(rules);
        this.rules = rules;
        this.suggestions = new ModSuggestions(commands, rules);
    }

    /**
     * Команда разбора собирается на каждый вызов: {@link dev.noro.agent.core.CaseMode}
     * появляется только со стартом канала, а дерево команд строится раньше.
     */
    private dev.noro.agent.core.CaseCommands caseCmd() {
        return new dev.noro.agent.core.CaseCommands(
                moderation.cases(), moderation.applier().bridge(), moderation.events(), freezeCmd);
    }

    void register(CommandDispatcher<CommandSourceStack> dispatcher) {
        AgentRuntime.LOG.info("Registering moderation commands in Brigadier: ban, serverban, mute, warn, unban, unmute, history, freeze, unfreeze, report, check, rules, rule, vanish");
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
        dispatcher.register(target("check", "noro.mod.punish.view", (source, name) ->
                checkCmd.check(new ModSender(source), new String[] {name}, null)));

        // freeze
        dispatcher.register(Commands.literal("freeze")
                .requires(source -> allowed(source, "noro.mod.freeze"))
                .then(Commands.argument("player", StringArgumentType.word())
                        .suggests((context, builder) -> SharedSuggestionProvider.suggest(onlineNames(context.getSource()), builder))
                        .executes(context -> {
                            freezeCmd.freeze(new ModSender(context.getSource()), new String[] { StringArgumentType.getString(context, "player") }, null);
                            return 1;
                        })
                        .then(Commands.argument("reason", StringArgumentType.greedyString())
                                .executes(context -> {
                                    String player = StringArgumentType.getString(context, "player");
                                    String reason = StringArgumentType.getString(context, "reason");
                                    freezeCmd.freeze(new ModSender(context.getSource()), new String[] { player, reason }, null);
                                    return 1;
                                }))));

        // unfreeze
        dispatcher.register(target("unfreeze", "noro.mod.freeze", (source, name) ->
                freezeCmd.unfreeze(new ModSender(source), new String[] {name}, null)));

        // case: клик по меню разбора приходит сюда же, что и набранная команда
        dispatcher.register(Commands.literal("case")
                .requires(source -> allowed(source, "noro.mod.cases.view"))
                .executes(context -> {
                    caseCmd().execute(new ModSender(context.getSource()), new String[0], null);
                    return 1;
                })
                .then(Commands.argument("args", StringArgumentType.greedyString())
                        .executes(context -> {
                            String raw = StringArgumentType.getString(context, "args");
                            caseCmd().execute(
                                    new ModSender(context.getSource()), raw.trim().split("\\s+"), null);
                            return 1;
                        })));

        // report
        dispatcher.register(Commands.literal("report")
                .then(Commands.argument("player", StringArgumentType.word())
                        .suggests((context, builder) -> SharedSuggestionProvider.suggest(onlineNames(context.getSource()), builder))
                        .then(Commands.argument("reason", StringArgumentType.greedyString())
                                .executes(context -> {
                                    String player = StringArgumentType.getString(context, "player");
                                    String reason = StringArgumentType.getString(context, "reason");
                                    reportCmd.execute(new ModSender(context.getSource()), new String[] { player, reason }, null);
                                    return 1;
                                }))));

        // rules / rule
        LiteralArgumentBuilder<CommandSourceStack> rulesNode = Commands.literal("rules")
                .executes(context -> {
                    ruleCmds.rules(new ModSender(context.getSource()), new String[0], null);
                    return 1;
                })
                .then(Commands.argument("code", StringArgumentType.word())
                        .suggests((context, builder) -> SharedSuggestionProvider.suggest(rules.matching(""), builder))
                        .executes(context -> {
                            String code = StringArgumentType.getString(context, "code");
                            ruleCmds.rules(new ModSender(context.getSource()), new String[] { code }, null);
                            return 1;
                        }));
        dispatcher.register(rulesNode);

        LiteralArgumentBuilder<CommandSourceStack> ruleNode = Commands.literal("rule")
                .executes(context -> {
                    ruleCmds.rules(new ModSender(context.getSource()), new String[0], null);
                    return 1;
                })
                .then(Commands.argument("code", StringArgumentType.word())
                        .suggests((context, builder) -> SharedSuggestionProvider.suggest(rules.matching(""), builder))
                        .executes(context -> {
                            String code = StringArgumentType.getString(context, "code");
                            ruleCmds.rules(new ModSender(context.getSource()), new String[] { code }, null);
                            return 1;
                        }));
        dispatcher.register(ruleNode);

        dispatcher.register(vanishCommand("vanish"));
        dispatcher.register(vanishCommand("v"));
    }

    private LiteralArgumentBuilder<CommandSourceStack> vanishCommand(String name) {
        return Commands.literal(name)
                .executes(context -> {
                    CommandSourceStack source = context.getSource();
                    if (source.getEntity() instanceof net.minecraft.server.level.ServerPlayer player) {
                        ModSender sender = new ModSender(source);
                        boolean canVanish = sender.has(ModVanishManager.PERM_USE);
                        boolean caseOnly = sender.has("noro.mod.vanish.case_only");
                        boolean hasActiveCase = ModModeration.getInstance().cases() != null && ModModeration.getInstance().cases().session(player.getUUID()).isPresent();

                        if (canVanish || (caseOnly && hasActiveCase)) {
                            ModVanishManager.getInstance().toggleVanish(player, null);
                        } else if (caseOnly) {
                            ModText.send(player, dev.noro.agent.core.AgentStrings.get(null, "vanish_case_only"));
                        } else {
                            ModText.send(player, dev.noro.agent.core.AgentStrings.get(null, "no_perm_view"));
                        }
                    }
                    return 1;
                })
                .then(Commands.literal("list")
                        .executes(context -> {
                            if (context.getSource().getEntity() instanceof net.minecraft.server.level.ServerPlayer player) {
                                ModVanishManager.getInstance().sendVanishList(player);
                            }
                            return 1;
                        }));
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
