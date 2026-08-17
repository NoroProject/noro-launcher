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
public final class Moderation implements AgentLink.Listener {

    private final ModerationClient client;
    private final MuteRegistry mutes = new MuteRegistry();
    private final Logger log;

    /** Читают из игрового потока на каждый отказ, пишут при обновлении с мастера. */
    private volatile MessageTemplates templates = MessageTemplates.defaults();

    private volatile PunishmentApplier applier;

    public Moderation(ModerationClient client, Logger log) {
        this.client = client;
        this.log = log;
    }

    /**
     * Подключить игру. До этого момента наказания применять некуда — сервер
     * ещё не запущен, и все входящие кадры уходят в лог.
     */
    public void attach(GameBridge bridge) {
        this.applier = new PunishmentApplier(bridge, mutes, this, log);
        refreshTemplates();
    }

    public ModerationClient client() {
        return client;
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

    /** Текст отказа замученному — то, что он видит вместо своего сообщения. */
    public String muteNotice(UUID uuid, String playerName) {
        PunishmentInfo mute = mutes.active(uuid);
        return mute == null ? null : MessageRender.render(templates.screen(mute), mute, playerName);
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
}
