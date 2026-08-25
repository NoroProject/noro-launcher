package dev.noro.agent.core;

import dev.noro.agent.core.automod.ChatFilters;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.slf4j.Logger;

/**
 * Модерация на этом сервере: муты, шаблоны, фильтры чата и применение наказаний.
 */
public final class Moderation implements AgentLink.Listener, AutoCloseable {

    private final ModerationClient client;
    private final MasterClient master;
    private final RuleCatalog rules;
    private final MuteRegistry mutes = new MuteRegistry();
    private final MuteSync sync;
    private final MaintenanceCountdown maintenanceCountdown;
    private final ChatFilters chatFilters;
    /** Короткое окно чата: срез уезжает в дело, когда появился повод. */
    private final ChatRing chatRing = new ChatRing();
    private final Logger log;
    /** Разбор жалоб в игре. Появляется вместе с мостом: без игры он бессмыслен. */
    private CaseMode cases;
    private AgentEvents events;

    /** Читают из игрового потока на каждый отказ, пишут при обновлении с мастера. */
    private volatile MessageTemplates templates = MessageTemplates.defaults();

    private volatile PunishmentApplier applier;

    /** Кто перечитывает профили. Ставит платформа: применение — её дело. */
    private volatile ProfileRefresher refresher;

    public Moderation(ModerationClient client, MasterClient master, RuleCatalog rules, Logger log) {
        this.client = client;
        this.master = master;
        this.rules = rules;
        this.log = log;
        this.sync = new MuteSync(master, mutes, log);
        this.maintenanceCountdown = new MaintenanceCountdown(log);
        this.chatFilters = new ChatFilters(master.http(), log);
    }

    public void attach(GameBridge bridge) {
        this.applier = new PunishmentApplier(bridge, mutes, this, log);
        refreshTemplates();
        sync.start();
        chatFilters.refresh();
        NoroAgentApi.attachMutes((uuid, actionbar) -> muteNotice(uuid, nameOf(uuid), actionbar));
    }

    /**
     * Подключить разбор жалоб. Отдельно от {@link #attach}: события канала
     * нужны разбору, а канал открывается уже после моста.
     */
    public void attachCases(GameBridge bridge, AgentEvents events, CaseMode.VanishBridge vanish) {
        this.events = events;
        this.cases = new CaseMode(bridge, events, chatRing, vanish);
    }

    public CaseMode cases() {
        return cases;
    }

    /** События канала: их шлёт разбор, когда модератор что-то делает. */
    public AgentEvents events() {
        return events;
    }

    public ChatRing chatRing() {
        return chatRing;
    }

    private static String nameOf(UUID uuid) {
        PlayerProfile profile = NoroAgentApi.profile(uuid);
        return profile == null ? "" : profile.username();
    }

    public void attachRefresher(ProfileRefresher refresher) {
        this.refresher = refresher;
    }

    public ModerationClient client() {
        return client;
    }

    public MasterClient master() {
        return master;
    }

    public RuleCatalog rules() {
        return rules;
    }

    public MuteRegistry mutes() {
        return mutes;
    }

    public MessageTemplates templates() {
        return templates;
    }

    public PunishmentApplier applier() {
        return applier;
    }

    public MaintenanceCountdown maintenanceCountdown() {
        return maintenanceCountdown;
    }

    public ChatFilters chatFilters() {
        return chatFilters;
    }

    public ChatFilters.Result checkChatMessage(UUID uuid, String text) {
        ChatFilters.Result res = chatFilters.check(uuid, text);
        if (res.action() == ChatFilters.Action.ALLOW) {
            return res;
        }

        // Сохраняем доказательство срабатывания в automod_triggers на мастере
        chatFilters.recordTrigger(uuid, res.filterType(), res.action().name().toLowerCase(), res.triggerMessage());

        String playerName = nameOf(uuid);

        if (res.action() == ChatFilters.Action.PUNISH) {
            chatFilters.issueAutoPunishment(uuid, res.filterType(), rules);
        } else if (res.action() == ChatFilters.Action.ESCALATE) {
            if (applier != null) {
                applier.bridge().announceToPermission("noro.mod.staff.notify", "#f87171[AutoMod ESCALATE: " + res.filterType() + "] " + playerName + ": " + res.triggerMessage());
            }
            chatFilters.issueAutoPunishment(uuid, res.filterType(), rules);
        } else if (res.action() == ChatFilters.Action.SHADOW) {
            if (applier != null) {
                applier.bridge().announceToPermission("noro.mod.staff.notify", "#fbbf24[AutoMod SHADOW: " + res.filterType() + "] " + playerName + ": " + res.triggerMessage());
            }
        } else if (res.action() == ChatFilters.Action.DENY && "escalate".equalsIgnoreCase(res.mode())) {
            if (applier != null) {
                applier.bridge().announceToPermission("noro.mod.staff.notify", "#f87171[AutoMod WARN: " + res.filterType() + "] " + playerName + ": " + res.triggerMessage());
            }
        }
        return res;
    }

    public String muteNotice(UUID uuid, String playerName, boolean actionbar) {
        if (mutes.stale(uuid, MuteSync.FRESH)) {
            sync.refresh(uuid);
        }
        PunishmentInfo mute = mutes.active(uuid);
        if (mute == null) {
            return null;
        }
        String template = actionbar ? templates.actionbar(mute) : templates.screen(mute);
        return render(template, mute, playerName);
    }

    public String render(String template, PunishmentInfo punishment, String playerName) {
        String title = rules == null ? null : rules.titleFor(punishment.ruleCode());
        return MessageRender.render(template, punishment, playerName, title, templates.rulesUrl());
    }

    public String reasonForRule(String ruleCode) {
        if (ruleCode == null || ruleCode.isBlank()) {
            return "";
        }
        String code = ruleCode.startsWith("@") ? ruleCode.substring(1) : ruleCode;
        String own = rules == null ? null : rules.punishReasonFor(code);
        if (own != null && !own.isBlank()) {
            return own;
        }
        String title = rules == null ? null : rules.titleFor(code);
        return templates.reasonByRule()
                .replace("{rule}", code)
                .replace("{rule_title}", title == null || title.isBlank() ? code : title);
    }

    public void refreshTemplates() {
        CompletableFuture.runAsync(() -> templates = client.messages());
    }

    @Override
    public void onPunished(UUID target, String targetName, PunishmentInfo punishment) {
        PunishmentApplier current = applier;
        if (current == null) {
            log.debug("Punishment {} arrived before the server was ready", punishment.id());
            return;
        }
        current.apply(target, targetName, punishment);
    }

    @Override
    public void onRevoked(UUID target, String targetName, String kind, String actorLabel) {
        PunishmentApplier current = applier;
        if (current != null) {
            current.lift(target, kind);
        }
    }

    @Override
    public void onMessagesChanged() {
        refreshTemplates();
    }

    @Override
    public void onFiltersChanged() {
        chatFilters.refresh();
    }

    @Override
    public void onProfileChanged(UUID uuid) {
        ProfileRefresher current = refresher;
        if (current == null) {
            return;
        }
        if (uuid == null) {
            current.refreshAll();
        } else {
            current.refresh(uuid);
        }
    }

    @Override
    public void onKick(UUID target, String message) {
        PunishmentApplier current = applier;
        if (current != null && target != null) {
            current.bridge().kick(target, message);
        }
    }

    @Override
    public void onTell(UUID target, String message) {
        PunishmentApplier current = applier;
        if (current != null && target != null) {
            current.bridge().tell(target, message);
        }
    }

    @Override
    public void onAnnounce(String message) {
        PunishmentApplier current = applier;
        if (current != null && message != null) {
            current.bridge().announce(message);
        }
    }

    @Override
    public void onMaintenanceStart(int countdownSeconds, String reason) {
        GameBridge bridge = applier == null ? null : applier.bridge();
        maintenanceCountdown.start(countdownSeconds, reason, bridge, templates);
    }

    @Override
    public void onMaintenanceCancel() {
        GameBridge bridge = applier == null ? null : applier.bridge();
        maintenanceCountdown.cancel(bridge);
    }

    @Override
    public void onRestartNotice(int seconds, String reason) {
        GameBridge bridge = applier == null ? null : applier.bridge();
        maintenanceCountdown.start(seconds, "[RESTART] " + (reason == null ? "Planned restart" : reason), bridge, templates);
    }

    @Override
    public void onCaseAssigned(CaseSession session, UUID moderator) {
        if (cases != null && moderator != null) {
            cases.assigned(session, moderator, langOf(moderator));
        }
    }

    @Override
    public void onCaseFinished(UUID moderator, UUID caseId, boolean closed) {
        if (cases != null && moderator != null && caseId != null) {
            cases.finished(moderator, caseId, closed, langOf(moderator));
        }
    }

    @Override
    public void onCaseChatRequest(UUID caseId, int beforeSecs) {
        if (cases != null) {
            cases.sendChatSlice(caseId, beforeSecs);
        }
    }

    @Override
    public void onCaseInventoryRequest(UUID caseId, UUID target) {
        if (cases != null) {
            cases.sendInventory(caseId, target, null);
        }
    }

    /** Язык игрока: профиль знает его, иначе английский. */
    private static String langOf(UUID uuid) {
        PlayerProfile profile = NoroAgentApi.profile(uuid);
        return profile == null || profile.locale() == null ? "en" : profile.locale();
    }

    @Override
    public void close() {
        sync.close();
        maintenanceCountdown.close();
    }
}
