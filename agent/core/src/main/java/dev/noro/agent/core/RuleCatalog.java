package dev.noro.agent.core;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

/**
 * Коды правил свода и лимиты сроков — для автодополнения в командах.
 *
 * <p>Без подсказки код правила приходится помнить наизусть, а без правила
 * мастер откажет всем, у кого нет {@code noro.mod.punish.bypass}. Список
 * тянется один раз при старте и живёт в памяти: свод меняется реже, чем
 * раз в сессию сервера.
 */
public final class RuleCatalog {

    private final MasterHttp http;
    private volatile List<String> codes = List.of();
    private volatile Map<String, List<String>> ruleDurations = Map.of();
    private volatile Map<String, String> ruleTitles = Map.of();
    private volatile Map<String, String> rulePunishReasons = Map.of();

    public RuleCatalog(MasterHttp http) {
        this.http = http;
    }

    /** Обновить фоном: старт сервера не должен ждать сеть ради подсказки. */
    public void refresh() {
        CompletableFuture.runAsync(() -> {
            try {
                RulesData data = http.get("/api/agent/rules", RulesData.class).orElse(null);
                if (data == null || data.rules == null) {
                    return;
                }
                Map<String, String> idToCode = new HashMap<>();
                Map<String, String> codeToTitle = new HashMap<>();
                Map<String, String> codeToReason = new HashMap<>();
                List<String> loadedCodes = new ArrayList<>();
                for (Rule rule : data.rules) {
                    if (rule.id != null && rule.code != null) {
                        idToCode.put(rule.id, rule.code);
                        loadedCodes.add(rule.code);
                        if (rule.title != null) {
                            codeToTitle.put(rule.code, rule.title);
                        }
                        if (rule.punishReason != null && !rule.punishReason.isBlank()) {
                            codeToReason.put(rule.code, rule.punishReason);
                        }
                    }
                }

                Map<String, List<String>> durationsMap = new HashMap<>();
                if (data.sanctions != null) {
                    for (Sanction sanction : data.sanctions) {
                        String code = idToCode.get(sanction.ruleId);
                        if (code == null) {
                            continue;
                        }
                        List<String> list = durationsMap.computeIfAbsent(code, k -> new ArrayList<>());
                        if (sanction.minMinutes != null) {
                            String formatted = DurationArg.format(sanction.minMinutes);
                            if (!list.contains(formatted)) list.add(formatted);
                        }
                        if (sanction.maxMinutes != null) {
                            String formatted = DurationArg.format(sanction.maxMinutes);
                            if (!list.contains(formatted)) list.add(formatted);
                        }
                        if (sanction.minMinutes == null && sanction.maxMinutes == null) {
                            if (!list.contains("perm")) list.add("perm");
                        }
                    }
                }

                codes = List.copyOf(loadedCodes);
                ruleDurations = Map.copyOf(durationsMap);
                ruleTitles = Map.copyOf(codeToTitle);
                rulePunishReasons = Map.copyOf(codeToReason);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            } catch (Exception ignored) {
                // Свод не догрузился — команды работают, только без подсказки.
            }
        });
    }

    /** Коды правил, всегда начинающиеся с @. Пустой ввод даёт весь список с @. */
    public List<String> matching(String prefix) {
        String raw = prefix == null ? "" : prefix.trim();
        String clean = raw.startsWith("@") ? raw.substring(1) : raw;
        String lower = clean.toLowerCase(java.util.Locale.ROOT);
        List<String> out = new ArrayList<>();
        for (String code : codes) {
            if (code.toLowerCase(java.util.Locale.ROOT).startsWith(lower)) {
                out.add("@" + code);
            }
        }
        return out;
    }

    /** Название правила по коду (1.1 -> "Уважение к другим игрокам"). */
    public String titleFor(String ruleCode) {
        if (ruleCode == null) {
            return null;
        }
        String clean = ruleCode.startsWith("@") ? ruleCode.substring(1) : ruleCode;
        return ruleTitles.get(clean);
    }

    /**
     * Формулировка наказания, заданная у пункта свода и переведённая мастером.
     *
     * <p>Именно её видит игрок в бане. Заголовок правила для этого не годится:
     * «Уважение к другим игрокам» в качестве причины читается как похвала.
     *
     * @return {@code null}, если у пункта её не задали — тогда причину соберёт
     *     шаблон {@code reason_by_rule}
     */
    public String punishReasonFor(String ruleCode) {
        if (ruleCode == null) {
            return null;
        }
        String clean = ruleCode.startsWith("@") ? ruleCode.substring(1) : ruleCode;
        return rulePunishReasons.get(clean);
    }

    /** Рекомендованные мастером сроки (min/max из санкций) для конкретного правила. */
    public List<String> durationsFor(String ruleCode) {
        if (ruleCode == null) {
            return List.of();
        }
        String clean = ruleCode.startsWith("@") ? ruleCode.substring(1) : ruleCode;
        return ruleDurations.getOrDefault(clean, List.of());
    }

    /** Часть ответа {@code GET /api/agent/rules}, которая нужна агенту. */
    private static final class RulesData {
        List<Rule> rules;
        List<Sanction> sanctions;
    }

    private static final class Rule {
        String id;
        String code;
        String title;
        String punishReason;
    }

    private static final class Sanction {
        String ruleId;
        String kind;
        Long minMinutes;
        Long maxMinutes;
    }
}
