package dev.noro.agent.core;

import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.slf4j.Logger;

/**
 * Модерация на этом сервере: муты, шаблоны и применение наказаний.
 *
 * <p>Собирает вместе то, что иначе пришлось бы протаскивать по отдельности в
 * каждую команду и в каждый слушатель. Платформа подключает сюда свой
 * {@link GameBridge} и получает готовое поведение.
 *
 * <p>Она же — приёмник живого канала: кадр с мастера превращается в кик или
 * снятый мут ровно теми же вызовами, что и ответ на команду.
 */
public final class Moderation implements AgentLink.Listener, AutoCloseable {

    private final ModerationClient client;
    private final MasterClient master;
    private final RuleCatalog rules;
    private final MuteRegistry mutes = new MuteRegistry();
    private final MuteSync sync;
    private final Logger log;

    /** Читают из игрового потока на каждый отказ, пишут при обновлении с мастера. */
    private volatile MessageTemplates templates = MessageTemplates.defaults();

    private volatile PunishmentApplier applier;

    public Moderation(ModerationClient client, MasterClient master, RuleCatalog rules, Logger log) {
        this.client = client;
        this.master = master;
        this.rules = rules;
        this.log = log;
        this.sync = new MuteSync(master, mutes, log);
    }

    /**
     * Подключить игру. До этого момента наказания применять некуда — сервер
     * ещё не запущен, и все входящие кадры уходят в лог.
     */
    public void attach(GameBridge bridge) {
        this.applier = new PunishmentApplier(bridge, mutes, this, log);
        refreshTemplates();
        sync.start();
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

    /**
     * Текст отказа замученному — то, что он видит вместо своего сообщения.
     *
     * <p>Заодно повод сверить мут с мастером: игрок пишет в чат именно тогда,
     * когда считает себя размученным, и если кадр о снятии не дошёл, это
     * единственный момент, когда мы можем это заметить.
     *
     * @param actionbar короткая версия — над хотбаром одна строка без переносов
     */
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

    /** Подстановка с тем, что знает только агент: названием пункта свода. */
    public String render(String template, PunishmentInfo punishment, String playerName) {
        String title = rules == null ? null : rules.titleFor(punishment.ruleCode());
        return MessageRender.render(template, punishment, playerName, title, templates.rulesUrl());
    }

    /**
     * Причина по правилу, когда модератор её не написал.
     *
     * <p>Текст берётся из шаблона мастера, а не собирается здесь: формулировка —
     * дело того, кто правит тексты, и на разных серверах она разная.
     */
    public String reasonForRule(String ruleCode) {
        if (ruleCode == null || ruleCode.isBlank()) {
            return "";
        }
        String code = ruleCode.startsWith("@") ? ruleCode.substring(1) : ruleCode;
        // Своя формулировка пункта важнее общего шаблона: её писали именно для
        // этого нарушения и перевели вместе со сводом.
        String own = rules == null ? null : rules.punishReasonFor(code);
        if (own != null && !own.isBlank()) {
            return own;
        }
        String title = rules == null ? null : rules.titleFor(code);
        return templates.reasonByRule()
                .replace("{rule}", code)
                .replace("{rule_title}", title == null || title.isBlank() ? code : title);
    }

    /** Перечитать шаблоны с мастера. Фоном: старт сервера не ждёт сеть. */
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
    public void close() {
        sync.close();
    }
}
