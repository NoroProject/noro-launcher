package dev.noro.agent.core.automod;

import dev.noro.agent.core.DurationArg;
import dev.noro.agent.core.MasterHttp;
import dev.noro.agent.core.RuleCatalog;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArrayList;
import org.slf4j.Logger;

/**
 * Набор правил и принятие решений при проверке чата.
 */
public final class ChatFilters {

    public enum Action {
        ALLOW,
        DENY,
        PUNISH,
        SHADOW,
        ESCALATE
    }

    public record Result(Action action, String filterType, String mode, String triggerMessage) {}

    private static class TriggerTracker {
        private final List<Long> timestamps = new CopyOnWriteArrayList<>();

        int countAndAdd(long now, int windowSecs) {
            long cutoff = now - (windowSecs * 1000L);
            timestamps.removeIf(t -> t < cutoff);
            timestamps.add(now);
            return timestamps.size();
        }
    }

    private final Map<String, FilterConfig> configs = new ConcurrentHashMap<>();
    private final Map<String, TriggerTracker> playerTrackers = new ConcurrentHashMap<>();
    private final FloodRule floodRule = new FloodRule();
    private final MasterHttp http;
    private final Logger log;

    public ChatFilters(MasterHttp http, Logger log) {
        this.http = http;
        this.log = log;
    }

    public void refresh() {
        if (http == null) return;
        CompletableFuture.runAsync(() -> {
            try {
                FilterConfig[] loaded = http.get("/api/agent/chat-filters", FilterConfig[].class).orElse(null);
                if (loaded != null) {
                    configs.clear();
                    for (FilterConfig cfg : loaded) {
                        if (cfg != null && cfg.filterType() != null) {
                            configs.put(cfg.filterType(), cfg);
                        }
                    }
                    if (log != null) {
                        log.info("Loaded {} chat auto-mod filters from master", configs.size());
                    }
                }
            } catch (Exception e) {
                if (log != null) {
                    log.warn("Failed to load chat filters from master: {}", e.getMessage());
                }
            }
        });
    }

    public void updateConfig(String filterType, FilterConfig config) {
        if (filterType != null && config != null) {
            configs.put(filterType, config);
        }
    }

    public Result check(UUID playerUuid, String message) {
        if (message == null || message.isBlank()) {
            return new Result(Action.ALLOW, null, null, null);
        }

        FilterConfig adConfig = configs.get("ad");
        if (adConfig != null && adConfig.enabled() && AdRule.check(message, adConfig)) {
            return buildResult("ad", adConfig, message, playerUuid);
        }

        FilterConfig wordConfig = configs.get("word");
        if (wordConfig != null && wordConfig.enabled() && WordRule.check(message, wordConfig)) {
            return buildResult("word", wordConfig, message, playerUuid);
        }

        FilterConfig capsConfig = configs.get("caps");
        if (capsConfig != null && capsConfig.enabled() && CapsRule.check(message, capsConfig)) {
            return buildResult("caps", capsConfig, message, playerUuid);
        }

        FilterConfig floodConfig = configs.get("flood");
        if (floodConfig != null && floodConfig.enabled() && floodRule.check(playerUuid, message, floodConfig)) {
            return buildResult("flood", floodConfig, message, playerUuid);
        }

        return new Result(Action.ALLOW, null, null, null);
    }

    private Result buildResult(String filterType, FilterConfig config, String message, UUID playerUuid) {
        String mode = config.mode() == null ? "deny" : config.mode();
        if ("escalate".equalsIgnoreCase(mode)) {
            int window = config.windowSecs() <= 0 ? 60 : config.windowSecs();
            int maxMsgs = config.maxMessages() <= 0 ? 1 : config.maxMessages();
            String trackerKey = playerUuid + ":" + filterType;
            TriggerTracker tracker = playerTrackers.computeIfAbsent(trackerKey, k -> new TriggerTracker());
            int count = tracker.countAndAdd(System.currentTimeMillis(), window);
            if (count <= maxMsgs) {
                return new Result(Action.DENY, filterType, mode, message);
            } else {
                return new Result(Action.ESCALATE, filterType, mode, message);
            }
        }

        Action action = switch (mode) {
            case "punish" -> Action.PUNISH;
            case "shadow" -> Action.SHADOW;
            default -> Action.DENY;
        };
        return new Result(action, filterType, mode, message);
    }

    public void recordTrigger(UUID playerUuid, String filterType, String action, String triggerText) {
        if (http == null || playerUuid == null) return;
        CompletableFuture.runAsync(() -> {
            try {
                Map<String, Object> body = Map.of(
                        "player_uuid", playerUuid.toString(),
                        "filter_type", filterType,
                        "mode", action,
                        "trigger_text", triggerText
                );
                http.post("/api/agent/automod-triggers", body);
            } catch (Exception e) {
                if (log != null) {
                    log.debug("Failed to record automod trigger: {}", e.getMessage());
                }
            }
        });
    }

    public void issueAutoPunishment(UUID targetUuid, String filterType, RuleCatalog ruleCatalog) {
        if (http == null) return;
        FilterConfig cfg = configs.get(filterType);
        String ruleCode = cfg != null ? cfg.ruleCode() : null;

        String reason;
        int durationSecs = 3600;
        if (ruleCode != null && !ruleCode.isBlank()) {
            if (ruleCatalog != null) {
                String own = ruleCatalog.punishReasonFor(ruleCode);
                String title = ruleCatalog.titleFor(ruleCode);
                reason = own != null && !own.isBlank() ? own : (title != null && !title.isBlank() ? title : "Violation of rule " + ruleCode);
                List<String> durations = ruleCatalog.durationsFor(ruleCode);
                if (durations != null && !durations.isEmpty()) {
                    Long parsedMins = DurationArg.parse(durations.get(0));
                    if (parsedMins != null && parsedMins > 0) {
                        durationSecs = (int) (parsedMins * 60);
                    }
                }
            } else {
                reason = "Violation of rule " + ruleCode;
            }
        } else {
            reason = "AutoMod violation [" + filterType + "]";
        }

        try {
            // Срок — в минутах: поле мастера называется `minutes`, и
            // `duration_seconds` он просто не видел. Незамеченное поле у него
            // значит «навсегда», то есть каждый автомут был вечным.
            Map<String, Object> payload = new java.util.HashMap<>();
            payload.put("kind", "mute");
            payload.put("reason", reason);
            payload.put("minutes", Math.max(1, durationSecs / 60));
            // Пустую строку не шлём: по ней правило не найдётся, а без правила
            // мастер не применит к наказанию никаких рамок.
            if (ruleCode != null && !ruleCode.isBlank()) {
                payload.put("rule_code", ruleCode);
            }
            http.post("/api/agent/players/" + targetUuid + "/punishments", payload);
        } catch (Exception e) {
            if (log != null) {
                log.warn("Failed to issue automod punishment to {}: {}", targetUuid, e.getMessage());
            }
        }
    }
}
