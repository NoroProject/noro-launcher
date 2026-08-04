package dev.noro.agent.core;

import java.util.Optional;
import java.util.UUID;
import org.slf4j.Logger;

/**
 * Решение о входе игрока — один раз для всех трёх платформ.
 *
 * <p>Само право считает мастер и отдаёт готовым флагом {@code allowed}; здесь
 * остаётся то, о чём мастер не знает: что делать при 404 и при обрыве связи.
 * Эти два случая и надо было свести в одно место, иначе Paper, NeoForge и
 * Fabric разъедутся в поведении на ровном месте.
 */
public final class AccessGate {

    private AccessGate() {}

    /**
     * @param profile профиль, если мастер ответил — чтобы не ходить за ролями
     *                вторым запросом
     */
    public record Decision(boolean allowed, String message, PlayerProfile profile) {

        static Decision allow(PlayerProfile profile) {
            return new Decision(true, null, profile);
        }

        static Decision deny(String message) {
            return new Decision(false, message, null);
        }
    }

    public static Decision check(MasterClient client, AgentConfig config, UUID mcUuid, Logger log) {
        try {
            Optional<PlayerProfile> found = client.player(mcUuid);
            if (found.isEmpty()) {
                // Настоящая граница доступа — «есть аккаунт на мастере».
                // Ключи и лаунчер её не задают, а вот это — задаёт.
                return Decision.deny("No account on this network. Sign in through the launcher first.");
            }
            PlayerProfile profile = found.get();
            if (profile.allowed()) {
                return Decision.allow(profile);
            }
            return Decision.deny(profile.banned()
                    ? "You are banned on this network."
                    : "You do not have access to this server.");
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return failure(config, log, "interrupted");
        } catch (Exception e) {
            return failure(config, log, e.getMessage());
        }
    }

    private static Decision failure(AgentConfig config, Logger log, String reason) {
        if (config.denyOnMasterError()) {
            log.warn("Master unreachable, denying login: {}", reason);
            return Decision.deny("Authentication service is unavailable. Try again in a minute.");
        }
        // Открытый режим включают осознанно: сервер переживает падение мастера,
        // но на это время пускает и тех, кого мастер бы не пустил.
        log.warn("Master unreachable, allowing login (deny-on-master-error=false): {}", reason);
        return Decision.allow(null);
    }
}
