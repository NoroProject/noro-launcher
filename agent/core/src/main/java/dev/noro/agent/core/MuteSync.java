package dev.noro.agent.core;

import java.time.Duration;
import java.util.UUID;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import org.slf4j.Logger;

/**
 * Сверка мутов с мастером — страховка живого канала.
 *
 * <p>Канал доставляет снятие мгновенно, но ровно до первого потерянного кадра:
 * он мог уйти, пока агент переподключался, или пока сервер лежал. Для игрока это
 * выглядит хуже всего остального — он размучен на сайте и молчит в игре, и
 * объяснить это в чате некому.
 *
 * <p>Спрашиваем только про замученных и только их профиль: на пустом реестре
 * задача не делает ни одного запроса, а на полном сервере запросов столько,
 * сколько замученных игроков, а не сколько игроков вообще.
 */
final class MuteSync implements AutoCloseable {

    /** Раз в столько сверяем сами. Канал быстрее, это подстраховка. */
    private static final Duration PERIOD = Duration.ofSeconds(30);

    /** Насколько свежей считается запись при проверке по требованию. */
    static final Duration FRESH = Duration.ofSeconds(10);

    private final MasterClient master;
    private final MuteRegistry mutes;
    private final Logger log;
    private final ScheduledExecutorService scheduler = Executors.newSingleThreadScheduledExecutor(task -> {
        Thread thread = new Thread(task, "noro-agent-mute-sync");
        thread.setDaemon(true);
        return thread;
    });

    MuteSync(MasterClient master, MuteRegistry mutes, Logger log) {
        this.master = master;
        this.mutes = mutes;
        this.log = log;
    }

    void start() {
        scheduler.scheduleWithFixedDelay(this::sweep, PERIOD.toSeconds(), PERIOD.toSeconds(), TimeUnit.SECONDS);
    }

    /** Проверить одного — когда его мут уже «староват», а он снова пишет в чат. */
    void refresh(UUID uuid) {
        scheduler.execute(() -> check(uuid));
    }

    private void sweep() {
        for (UUID uuid : mutes.mutedPlayers()) {
            check(uuid);
        }
    }

    private void check(UUID uuid) {
        try {
            PlayerProfile profile = master.player(uuid).orElse(null);
            if (profile == null) {
                return;
            }
            PunishmentInfo before = mutes.active(uuid);
            mutes.remember(uuid, profile.activeMute());
            if (before != null && profile.activeMute() == null) {
                // Расхождение кэша с мастером стоит видеть в логе: раз оно
                // случилось, значит кадр о снятии до нас не дошёл.
                log.info(
                        "Mute of {} is gone on master — chat is back (cached until {})",
                        profile.username(),
                        before.expiresAt());
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        } catch (Exception e) {
            // Мастер недоступен — оставляем как есть: снимать мут по обрыву связи
            // значит дать замученному слово ровно тогда, когда проверить нечем.
            log.debug("Cannot re-check mute of {}: {}", uuid, e.getMessage());
        }
    }

    @Override
    public void close() {
        scheduler.shutdownNow();
    }
}
