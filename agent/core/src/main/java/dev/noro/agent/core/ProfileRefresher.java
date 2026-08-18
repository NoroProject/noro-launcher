package dev.noro.agent.core;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.function.Supplier;
import org.slf4j.Logger;

/**
 * Перечитывание профиля живого игрока.
 *
 * <p>{@link ProfileCache} наполняется на входе и чистится на выходе, а между
 * ними обновляются одни муты. Из-за этого выданная на сайте роль не значила в
 * игре ничего до перезахода — притом что игрок не знает, что должен перезайти.
 *
 * <p>Что делать с новым профилем, знает платформа: разложить права надо в
 * Bukkit-attachment или в свою карту, а группы — в LuckPerms. Поэтому здесь
 * только сеть и решение «кого спрашивать», а применение приходит снаружи.
 *
 * <p>Всё уходит с общего пула: кадр прилетает из потока WebSocket, а держать в
 * нём запрос к мастеру значит задержать все остальные кадры канала.
 */
public final class ProfileRefresher {

    /** Что платформа делает с обновлённым профилем. */
    public interface Applier {
        void apply(UUID uuid, PlayerProfile profile);
    }

    private final MasterClient master;
    private final Supplier<Collection<UUID>> online;
    private final Applier applier;
    private final Logger log;

    public ProfileRefresher(
            MasterClient master, Supplier<Collection<UUID>> online, Applier applier, Logger log) {
        this.master = master;
        this.online = online;
        this.applier = applier;
        this.log = log;
    }

    /** Один игрок. Нет на сервере — обновлять нечего: профиль живёт с ним. */
    public void refresh(UUID uuid) {
        if (!online.get().contains(uuid)) {
            return;
        }
        CompletableFuture.runAsync(() -> {
            try {
                master.player(uuid).ifPresent(profile -> applier.apply(uuid, profile));
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            } catch (Exception e) {
                log.warn("Cannot refresh profile of {}: {}", uuid, e.getMessage());
            }
        });
    }

    /**
     * Все, кто в сети, — одним запросом.
     *
     * <p>Батчем, а не циклом: правка роли задевает каждого её носителя, и на
     * сервере с сотней игроков это была бы сотня запросов в одну секунду.
     */
    public void refreshAll() {
        List<UUID> uuids = new ArrayList<>(online.get());
        if (uuids.isEmpty()) {
            return;
        }
        CompletableFuture.runAsync(() -> {
            try {
                for (PlayerProfile profile : master.playersBatch(uuids)) {
                    applier.apply(profile.uuid(), profile);
                }
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            } catch (Exception e) {
                log.warn("Cannot refresh {} profiles: {}", uuids.size(), e.getMessage());
            }
        });
    }
}
