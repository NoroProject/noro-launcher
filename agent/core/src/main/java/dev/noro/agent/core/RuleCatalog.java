package dev.noro.agent.core;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;

/**
 * Коды правил свода — для автодополнения {@code @rule} в командах.
 *
 * <p>Без подсказки код правила приходится помнить наизусть, а без правила
 * мастер откажет всем, у кого нет {@code noro.mod.punish.bypass}. Поэтому
 * список тянется один раз при старте и живёт в памяти: свод меняется реже, чем
 * раз в сессию сервера.
 */
public final class RuleCatalog {

    private final MasterHttp http;
    private volatile List<String> codes = List.of();

    public RuleCatalog(MasterHttp http) {
        this.http = http;
    }

    /** Обновить фоном: старт сервера не должен ждать сеть ради подсказки. */
    public void refresh() {
        CompletableFuture.runAsync(() -> {
            try {
                Rules rules = http.get("/api/agent/rules", Rules.class).orElse(null);
                if (rules == null || rules.rules == null) {
                    return;
                }
                List<String> loaded = new ArrayList<>(rules.rules.size());
                for (Rule rule : rules.rules) {
                    if (rule.code != null) {
                        loaded.add(rule.code);
                    }
                }
                codes = List.copyOf(loaded);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            } catch (Exception ignored) {
                // Свод не догрузился — команды работают, только без подсказки.
            }
        });
    }

    /** Коды, подходящие под начало ввода. Пустой ввод даёт весь список. */
    public List<String> matching(String prefix) {
        String lower = prefix == null ? "" : prefix.toLowerCase(java.util.Locale.ROOT);
        List<String> out = new ArrayList<>();
        for (String code : codes) {
            if (code.toLowerCase(java.util.Locale.ROOT).startsWith(lower)) {
                out.add("@" + code);
            }
        }
        return out;
    }

    /** Часть ответа {@code GET /api/agent/rules}, которая нужна агенту. */
    private static final class Rules {
        List<Rule> rules;
    }

    private static final class Rule {
        String code;
    }
}
