package dev.noro.agent.mod;

import dev.noro.agent.core.AccessGate;
import dev.noro.agent.core.AgentConfig;
import dev.noro.agent.core.MasterClient;
import dev.noro.agent.core.NoroAgentApi;
import dev.noro.agent.core.PermissionSet;
import java.util.Collection;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Права игроков и каталог узлов на стороне мода.
 *
 * <p>Типов Minecraft здесь нет, поэтому нет и директив препроцессора: когда
 * именно звать {@link #load}, решает точка входа своего лоадера.
 *
 * <p>Карты именно конкурентные: пишет в них поток логина, а читает игровой —
 * на каждую проверку права.
 */
final class ModPermissions {

    private final MasterClient client;
    private final AgentConfig config;

    private final Map<UUID, PermissionSet> byPlayer = new ConcurrentHashMap<>();

    /**
     * Решение мастера, снятое ещё на логине. Вход в мир его забирает вместо
     * второго запроса за тем же самым.
     */
    private final Map<UUID, AccessGate.Decision> negotiated = new ConcurrentHashMap<>();

    ModPermissions(MasterClient client, AgentConfig config) {
        this.client = client;
        this.config = config;
    }

    /**
     * Ходит к мастеру и раскладывает права игрока. Блокирующий по замыслу:
     * звать его надо там, где логин обязан дождаться ответа, — иначе сервер
     * успеет прочитать права раньше, чем они появятся.
     */
    void load(UUID uuid) {
        AccessGate.Decision decision = AccessGate.check(client, config, uuid, AgentRuntime.LOG);
        if (decision.profile() != null) {
            byPlayer.put(uuid, PermissionSet.of(decision.profile().permissions()));
            NoroAgentApi.cache().remember(uuid, decision.profile());
        }
        negotiated.put(uuid, decision);
    }

    /**
     * Разложить уже прочитанный профиль. Отдельно от {@link #load}: там мы сами
     * идём к мастеру, а сюда профиль приносят — по кадру об изменении ролей.
     */
    void remember(UUID uuid, dev.noro.agent.core.PlayerProfile profile) {
        byPlayer.put(uuid, PermissionSet.of(profile.permissions()));
        NoroAgentApi.cache().remember(uuid, profile);
    }

    /** @return {@code null}, если логин прошёл мимо {@link #load} — тогда решение спрашивают заново */
    AccessGate.Decision takeDecision(UUID uuid) {
        return negotiated.remove(uuid);
    }

    /** Пустой набор, а не {@code null}: у неизвестного игрока просто нет прав. */
    PermissionSet of(UUID uuid) {
        return byPlayer.getOrDefault(uuid, PermissionSet.empty());
    }

    /**
     * Оборванный логин — единственный случай, когда запись остаётся: выхода не
     * было, значит и события выхода не будет. Следующая попытка того же игрока
     * её перезапишет, поэтому карта растёт не быстрее списка знакомых UUID.
     */
    void forget(UUID uuid) {
        byPlayer.remove(uuid);
        negotiated.remove(uuid);
    }

    void rememberNodes(Collection<String> names) {
        NoroAgentApi.permissionNodes().register(names);
    }

    /**
     * Подключает отправку каталога. Дальше он уходит сам — и на том, что успели
     * собрать с реестра лоадера, и на том, что позже заявят чужие моды: на
     * Fabric реестра нет вовсе, и заявка — единственный источник.
     */
    void report() {
        NoroAgentApi.permissionNodes().attach(this::send);
    }

    private void send(Collection<String> current) {
        if (current.isEmpty()) {
            return;
        }
        try {
            client.reportNodes(current);
            AgentRuntime.LOG.info("Reported {} permission nodes to master", current.size());
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        } catch (Exception e) {
            // Каталог — подсказка для админки, а не условие работы сервера.
            AgentRuntime.LOG.warn("Cannot report permission nodes: {}", e.getMessage());
        }
    }
}
