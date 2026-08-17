package dev.noro.agent.core;

import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.slf4j.Logger;

/**
 * Применение наказания к живому серверу: кик, мут, показ предупреждения.
 *
 * <p>Одна дорога для всех источников. Наказание доезжает сюда и кадром живого
 * канала, и ответом на команду в игре — модератор не должен видеть «выдано»
 * раньше, чем оно подействовало, а канал мог в этот момент переподключаться.
 * Поэтому применяем дважды и глушим повтор по id: дважды показанный варн
 * выглядит как два наказания.
 */
public final class PunishmentApplier {

    /** Сколько наказаний помним, чтобы не применить повторно. */
    private static final int SEEN = 256;

    private final GameBridge bridge;
    private final MuteRegistry mutes;
    private final Moderation moderation;
    private final Logger log;

    private final Set<UUID> seen = Collections.newSetFromMap(Collections.synchronizedMap(
            new LinkedHashMap<>(SEEN, 0.75f, true) {
                @Override
                protected boolean removeEldestEntry(Map.Entry<UUID, Boolean> eldest) {
                    return size() > SEEN;
                }
            }));

    public PunishmentApplier(GameBridge bridge, MuteRegistry mutes, Moderation moderation, Logger log) {
        this.bridge = bridge;
        this.mutes = mutes;
        this.moderation = moderation;
        this.log = log;
    }

    /** Наказание выдано: показать и применить. */
    public void apply(UUID target, String targetName, PunishmentInfo punishment) {
        if (punishment == null || punishment.id() == null || !seen.add(punishment.id())) {
            return;
        }
        MessageTemplates templates = moderation.templates();
        String text = moderation.render(templates.screen(punishment), punishment, targetName);

        if (punishment.disconnects()) {
            bridge.kick(target, text);
        } else if ("mute".equals(punishment.kind())) {
            mutes.remember(target, punishment);
            bridge.tell(target, text);
            // И над хотбаром: игрок в этот момент смотрит в мир, а не в чат.
            bridge.actionbar(target, moderation.render(templates.actionbar(punishment), punishment, targetName));
        } else {
            bridge.tell(target, text);
            bridge.actionbar(target, moderation.render(templates.actionbar(punishment), punishment, targetName));
            acknowledge(target, punishment);
        }

        String announcement = moderation.render(templates.broadcast(), punishment, targetName);
        if (!announcement.isBlank()) {
            bridge.announce(announcement);
        }
    }

    /** Наказание снято. Кика назад не бывает, а вот чат вернуть надо. */
    public void lift(UUID target, String kind) {
        if ("mute".equals(kind)) {
            mutes.forget(target);
            bridge.tell(target, "§aYou can speak again.");
        }
    }

    /**
     * Вход игрока: мут переносится в реестр, непрочитанные предупреждения
     * показываются и тут же подтверждаются.
     */
    public void greet(UUID uuid, PlayerProfile profile) {
        if (profile == null) {
            return;
        }
        mutes.remember(uuid, profile.activeMute());
        for (PunishmentInfo warn : profile.pendingWarns()) {
            seen.add(warn.id());
            bridge.tell(uuid, moderation.render(moderation.templates().warnNotice(), warn, profile.username()));
            acknowledge(uuid, warn);
        }
    }

    /**
     * Предупреждение засчитывается прочитанным только когда игрок его увидел.
     * Сеть здесь фоновая: показ уже случился, и ждать мастера незачем.
     */
    private void acknowledge(UUID player, PunishmentInfo warn) {
        CompletableFuture.runAsync(() -> {
            try {
                moderation.client().acknowledge(warn.id(), player);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            } catch (Exception e) {
                log.warn("Cannot mark warning {} as read: {}", warn.id(), e.getMessage());
            }
        });
    }
}
