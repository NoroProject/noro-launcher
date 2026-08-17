package dev.noro.agent.core;

import java.io.IOException;
import java.util.List;
import java.util.UUID;

/**
 * Наказания и шаблоны сообщений: то, что агент спрашивает у мастера от имени
 * модератора.
 *
 * <p>Права и рамки свода проверяет мастер, а не мы. Плагин мог бы проверить их
 * и сам, но тогда «кому сколько можно» жило бы в четырёх местах: на сайте и в
 * трёх реализациях агента.
 */
public final class ModerationClient {

    private final MasterHttp http;

    public ModerationClient(MasterHttp http) {
        this.http = http;
    }

    /**
     * Выдать наказание.
     *
     * @param actor MC UUID модератора; {@code null} — наказывает сам сервер
     * @throws MasterRefusedException отказ по существу — текст
     *     предназначен модератору и показывается ему дословно
     */
    public PunishmentInfo punish(UUID target, String kind, String reason, Long minutes, String ruleCode, UUID actor)
            throws IOException, InterruptedException {
        Punish body = new Punish(kind, reason, minutes, ruleCode, actor);
        return http.post("/api/agent/players/" + target + "/punishments", body, PunishmentInfo.class);
    }

    /** Снять наказание. Право {@code noro.mod.punish.revoke} проверяет мастер. */
    public void revoke(UUID punishmentId, UUID actor) throws IOException, InterruptedException {
        http.post("/api/agent/punishments/" + punishmentId + "/revoke", new Actor(actor));
    }

    /** Игрок прочитал предупреждение. Подтверждает он сам за себя. */
    public void acknowledge(UUID punishmentId, UUID player) throws IOException, InterruptedException {
        http.post("/api/agent/punishments/" + punishmentId + "/ack", new Actor(player));
    }

    /** Вся история игрока, включая снятое и истёкшее. */
    public List<PunishmentInfo> history(UUID target) throws IOException, InterruptedException {
        PunishmentInfo[] rows = http.get("/api/agent/players/" + target + "/punishments", PunishmentInfo[].class)
                .orElse(new PunishmentInfo[0]);
        return List.of(rows);
    }

    /**
     * Тексты экрана бана, отказа в чате и предупреждения.
     *
     * <p>Недоступность мастера здесь не должна затыкать сервер: без шаблонов
     * агент возьмёт встроенные и продолжит работать.
     */
    public MessageTemplates messages() {
        try {
            return http.get("/api/agent/messages", MessageTemplates.class)
                    .map(MessageTemplates::complete)
                    .orElseGet(MessageTemplates::defaults);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return MessageTemplates.defaults();
        } catch (Exception e) {
            return MessageTemplates.defaults();
        }
    }

    /** Тело выдачи. Имена полей уходят в snake_case политикой Gson. */
    private record Punish(String kind, String reason, Long minutes, String ruleCode, UUID actorUuid) {}

    /** Тело, где от агента нужен только тот, кто действует. */
    private record Actor(UUID actorUuid) {}
}
