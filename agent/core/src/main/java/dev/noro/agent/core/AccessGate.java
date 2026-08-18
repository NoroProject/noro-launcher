package dev.noro.agent.core;

import java.util.Optional;
import java.util.UUID;
import org.slf4j.Logger;

/**
 * Решение о входе игрока — один раз для всех трёх платформ.
 */
public final class AccessGate {

    private AccessGate() {}

    /** Почему не пустили. */
    public enum Reason {
        /** Бан сети: не пускает никуда. */
        NETWORK_BAN,
        /** Бан этой сборки: на остальных серверах доступ есть. */
        SERVER_BAN,
        /** Мастер такого игрока не знает — настоящая граница доступа. */
        NO_ACCOUNT,
        /** Аккаунт есть, доступа к этой сборке нет. */
        NO_ACCESS,
        /** Мастер недоступен, а сервер настроен никого не пускать вслепую. */
        MASTER_DOWN,
        /** Сервер на техническом обслуживании. */
        MAINTENANCE
    }

    public record Denial(Reason reason, PunishmentInfo punishment, String playerName) {}

    public record Decision(boolean allowed, Denial denial, PlayerProfile profile) {

        static Decision allow(PlayerProfile profile) {
            return new Decision(true, null, profile);
        }

        static Decision deny(Reason reason, PunishmentInfo punishment, String playerName) {
            return new Decision(false, new Denial(reason, punishment, playerName), null);
        }
    }

    public static Decision check(MasterClient client, AgentConfig config, UUID mcUuid, Logger log) {
        try {
            Optional<PlayerProfile> found = client.player(mcUuid);
            if (found.isEmpty()) {
                return Decision.deny(Reason.NO_ACCOUNT, null, null);
            }
            PlayerProfile profile = found.get();
            if (profile.allowed()) {
                return Decision.allow(profile);
            }
            return refuse(profile);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return failure(config, log, "interrupted");
        } catch (Exception e) {
            return failure(config, log, e.getMessage());
        }
    }

    private static Decision refuse(PlayerProfile profile) {
        if ("maintenance".equals(profile.denialReason())) {
            return Decision.deny(Reason.MAINTENANCE, null, profile.username());
        }
        PunishmentInfo ban = profile.activeBan();
        if (ban != null) {
            Reason reason =
                    "server_ban".equals(ban.kind()) ? Reason.SERVER_BAN : Reason.NETWORK_BAN;
            return Decision.deny(reason, ban, profile.username());
        }
        Reason reason = profile.banned() ? Reason.NETWORK_BAN : Reason.NO_ACCESS;
        return Decision.deny(reason, null, profile.username());
    }

    private static Decision failure(AgentConfig config, Logger log, String reason) {
        if (config.denyOnMasterError()) {
            log.warn("Master unreachable, denying login: {}", reason);
            return Decision.deny(Reason.MASTER_DOWN, null, null);
        }
        log.warn("Master unreachable, allowing login (deny-on-master-error=false): {}", reason);
        return Decision.allow(null);
    }
}
