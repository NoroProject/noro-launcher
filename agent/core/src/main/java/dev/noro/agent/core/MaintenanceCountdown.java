package dev.noro.agent.core;

import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.ScheduledFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import org.slf4j.Logger;

/**
 * Обратный отсчёт до включения техработ с оповещением игроков в чат/actionbar
 * и последующим отключением тех, у кого нет права noro.server.maintenance.bypass.
 */
public final class MaintenanceCountdown implements AutoCloseable {

    private final Logger log;
    private final ScheduledExecutorService timer = Executors.newSingleThreadScheduledExecutor(r -> {
        Thread t = new Thread(r, "noro-maintenance-countdown");
        t.setDaemon(true);
        return t;
    });

    private volatile boolean active;
    private volatile String currentReason = "";
    private volatile ScheduledFuture<?> currentTask;

    public MaintenanceCountdown(Logger log) {
        this.log = log;
    }

    public boolean active() {
        return active;
    }

    public String reason() {
        return currentReason;
    }

    public synchronized void start(int countdownSeconds, String reason, GameBridge bridge, MessageTemplates templates) {
        cancelTask();
        this.active = true;
        this.currentReason = reason == null ? "" : reason.trim();

        if (countdownSeconds <= 0) {
            log.info("Starting maintenance immediately, kicking players...");
            kickNonBypass(bridge, templates);
            return;
        }

        log.info("Scheduling maintenance countdown for {} seconds. Reason: {}", countdownSeconds, currentReason);
        AtomicInteger remaining = new AtomicInteger(countdownSeconds);

        currentTask = timer.scheduleAtFixedRate(() -> {
            try {
                int sec = remaining.getAndDecrement();
                if (sec <= 0) {
                    cancelTask();
                    kickNonBypass(bridge, templates);
                    return;
                }

                if (shouldAnnounce(sec)) {
                    String timeStr = formatSeconds(sec);
                    String announcement = AgentStrings.get("ru", "maintenance_countdown_announcement", timeStr, currentReason.isBlank() ? "Плановые работы" : currentReason);
                    if (bridge != null) {
                        bridge.announce(announcement);
                        if (sec <= 10) {
                            for (String name : bridge.onlineNames()) {
                                bridge.onlineUuid(name).ifPresent(u -> bridge.actionbar(u, announcement));
                            }
                        }
                    }
                }
            } catch (Exception e) {
                log.warn("Error during maintenance countdown tick: {}", e.getMessage());
            }
        }, 0, 1, TimeUnit.SECONDS);
    }

    public synchronized void cancel(GameBridge bridge) {
        cancelTask();
        if (active) {
            active = false;
            if (bridge != null) {
                bridge.announce("#4ade80[ТЕХРАБОТЫ] Техническое обслуживание отменено.");
            }
            log.info("Maintenance countdown cancelled.");
        }
    }

    private void cancelTask() {
        ScheduledFuture<?> task = currentTask;
        if (task != null) {
            task.cancel(false);
            currentTask = null;
        }
    }

    private boolean shouldAnnounce(int sec) {
        if (sec <= 5) return true;
        if (sec == 10 || sec == 15 || sec == 30 || sec == 45) return true;
        return sec % 60 == 0;
    }

    private String formatSeconds(int sec) {
        if (sec >= 60) {
            int mins = sec / 60;
            int remSec = sec % 60;
            if (remSec == 0) return mins + " мин.";
            return mins + " мин. " + remSec + " сек.";
        }
        return sec + " сек.";
    }

    private void kickNonBypass(GameBridge bridge, MessageTemplates templates) {
        if (bridge == null) return;
        String reasonStr = currentReason.isBlank() ? "Плановые работы" : currentReason;
        String kickMsg = templates != null && templates.maintenance() != null && !templates.maintenance().isBlank()
                ? MessageRender.render(templates.maintenance(), null, "", "", "")
                : "#f87171[MAINTENANCE] " + reasonStr;

        for (String name : bridge.onlineNames()) {
            bridge.onlineUuid(name).ifPresent(uuid -> {
                PlayerProfile profile = NoroAgentApi.profile(uuid);
                if (profile == null || !profile.hasPermission("noro.server.maintenance.bypass")) {
                    bridge.kick(uuid, kickMsg);
                }
            });
        }
    }

    @Override
    public void close() {
        cancelTask();
        timer.shutdownNow();
    }
}
