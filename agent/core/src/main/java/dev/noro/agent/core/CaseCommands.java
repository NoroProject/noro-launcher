package dev.noro.agent.core;

import java.util.List;
import java.util.Map;
import java.util.UUID;

/**
 * Команда {@code /case …} — то, во что превращается клик по меню разбора.
 *
 * <p>Каждое действие уходит кадром мастеру и попадает в ленту дела: разбор
 * должен читаться на сайте, пока он идёт, а не после того, как модератор
 * вспомнит, что делал.
 */
public final class CaseCommands {

    private final CaseMode mode;
    private final GameBridge game;
    private final AgentEvents events;
    private final FreezeCommand freeze;

    public CaseCommands(CaseMode mode, GameBridge game, AgentEvents events, FreezeCommand freeze) {
        this.mode = mode;
        this.game = game;
        this.events = events;
        this.freeze = freeze;
    }

    public void execute(CommandSender sender, String[] args, String lang) {
        UUID moderator = sender.uuid();
        if (moderator == null) {
            sender.reply(AgentStrings.get(lang, "case_console"));
            return;
        }
        if (args.length > 0 && args[0].equals("claim") && args.length > 1) {
            claim(sender, args[1], lang);
            return;
        }
        // Переключение между своими делами. Панель зовёт это, открывая
        // карточку: иначе «телепорт» уходил бы в то дело, которое агент
        // запомнил последним, а не в то, что модератор видит.
        if (args.length > 1 && args[0].equals("use")) {
            use(sender, args[1], lang);
            return;
        }

        CaseSession session = mode.session(moderator).orElse(null);
        if (session == null) {
            sender.reply(AgentStrings.get(lang, "case_none"));
            return;
        }

        String action = args.length > 0 ? args[0] : "menu";
        switch (action) {
            case "menu" -> sender.reply(CaseMenu.render(session, lang));
            case "tp" -> teleport(sender, session, args.length > 1 ? args[1] : "target", lang);
            case "back" -> back(sender, session, lang);
            case "chat" -> chat(sender, session, lang);
            case "inv" -> inventory(sender, session, lang);
            case "invsee" -> openContainer(sender, session, lang, false);
            case "ender" -> openContainer(sender, session, lang, true);
            case "watch" -> watch(sender, session, lang);
            case "freeze" -> freezeTarget(sender, session, lang);
            case "close", "release" -> sender.reply(AgentStrings.get(lang, "case_close_on_site"));
            default -> sender.reply(AgentStrings.get(lang, "case_usage"));
        }
    }

    private void use(CommandSender sender, String rawId, String lang) {
        try {
            if (mode.use(sender.uuid(), java.util.UUID.fromString(rawId))) {
                mode.session(sender.uuid())
                        .ifPresent(s -> sender.reply(CaseMenu.render(s, lang)));
                return;
            }
        } catch (IllegalArgumentException ignored) {
            // Ниже общий ответ: для модератора «не тот номер» и «дело не за
            // вами» — одно и то же, разбираться в разнице ему незачем.
        }
        sender.reply(AgentStrings.get(lang, "case_none"));
    }

    private void claim(CommandSender sender, String rawId, String lang) {
        try {
            events.caseClaim(UUID.fromString(rawId), sender.uuid());
            sender.reply(AgentStrings.get(lang, "case_claim_sent"));
        } catch (IllegalArgumentException e) {
            sender.reply(AgentStrings.get(lang, "case_bad_id"));
        }
    }

    private void teleport(CommandSender sender, CaseSession session, String where, String lang) {
        boolean done = switch (where) {
            case "place" -> session.hasPlace()
                    && game.teleport(sender.uuid(), session.world(), session.x(), session.y(), session.z());
            case "reporter" -> session.reporter() != null && game.teleportTo(sender.uuid(), session.reporter());
            default -> game.teleportTo(sender.uuid(), session.target());
        };
        sender.reply(AgentStrings.get(lang, done ? "case_teleported" : "case_teleport_failed"));
        if (done) {
            events.caseAction(session.caseId(), sender.uuid(), "teleport", Map.of("to", where));
        }
    }

    private void back(CommandSender sender, CaseSession session, String lang) {
        GameBridge.Position origin = session.origin();
        boolean done = origin != null
                && game.teleport(sender.uuid(), origin.world(), origin.x(), origin.y(), origin.z());
        sender.reply(AgentStrings.get(lang, done ? "case_teleported" : "case_teleport_failed"));
    }

    private void chat(CommandSender sender, CaseSession session, String lang) {
        List<ChatRing.Entry> slice = mode.chat().slice(java.time.Duration.ofMinutes(5));
        if (slice.isEmpty()) {
            sender.reply(AgentStrings.get(lang, "case_chat_empty"));
            return;
        }
        StringBuilder out = new StringBuilder(AgentStrings.get(lang, "case_chat_title"));
        for (ChatRing.Entry entry : slice) {
            out.append("\n#8b8b8b").append(entry.senderName()).append(": #e6e6e6").append(entry.content());
        }
        sender.reply(out.toString());
        // Срез, который модератор посмотрел, уезжает и в дело: то, на что он
        // опирался при решении, должно остаться в разборе.
        events.caseChatSlice(session.caseId(), slice);
    }

    private void inventory(CommandSender sender, CaseSession session, String lang) {
        List<GameBridge.Slot> items = game.inventory(session.target());
        if (items.isEmpty()) {
            sender.reply(AgentStrings.get(lang, "case_inventory_empty"));
            return;
        }
        String text = items.stream().map(GameBridge.Slot::label).collect(java.util.stream.Collectors.joining(", "));
        sender.reply(AgentStrings.get(lang, "case_inventory_title") + "\n#8b8b8b" + text);
        events.caseInventory(session.caseId(), sender.uuid(), items);
    }

    /**
     * Открыть инвентарь цели контейнером.
     *
     * <p>В отличие от снимка, это живые слоты: из них можно забрать улику и
     * вернуть унесённое. Снимок при этом всё равно уходит в дело — иначе в
     * ленте не останется следа, что модератор туда вообще заглядывал.
     */
    private void openContainer(
            CommandSender sender, CaseSession session, String lang, boolean ender) {
        boolean opened = ender
                ? game.openEnderChest(sender.uuid(), session.target())
                : game.openInventory(sender.uuid(), session.target());
        if (!opened) {
            sender.reply(AgentStrings.get(lang, "case_inventory_unsupported"));
            return;
        }
        events.caseAction(
                session.caseId(),
                sender.uuid(),
                ender ? "ender_open" : "inventory_open",
                java.util.Map.of("target", session.targetName()));
    }

    private void watch(CommandSender sender, CaseSession session, String lang) {
        boolean start = !session.watching();
        boolean done = start ? game.spectate(sender.uuid(), session.target()) : game.stopSpectate(sender.uuid());
        if (!done) {
            sender.reply(AgentStrings.get(lang, "case_watch_unsupported"));
            return;
        }
        mode.session(sender.uuid()).ifPresent(current -> mode.replace(sender.uuid(), current.watching(start)));
        sender.reply(AgentStrings.get(lang, start ? "case_watch_on" : "case_watch_off"));
        events.caseAction(session.caseId(), sender.uuid(), start ? "watch_start" : "watch_stop", Map.of());
    }

    private void freezeTarget(CommandSender sender, CaseSession session, String lang) {
        freeze.freeze(sender, new String[] {session.targetName(), AgentStrings.get(lang, "case_freeze_reason")}, lang);
        events.caseAction(session.caseId(), sender.uuid(), "freeze", Map.of("target", session.targetName()));
    }
}
