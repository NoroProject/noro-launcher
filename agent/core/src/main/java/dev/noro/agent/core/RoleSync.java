package dev.noro.agent.core;

import java.util.ArrayList;
import java.util.Collection;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import net.luckperms.api.LuckPerms;
import net.luckperms.api.model.user.User;
import net.luckperms.api.node.NodeType;
import net.luckperms.api.node.types.InheritanceNode;
import org.slf4j.Logger;

/**
 * Проекция ролей мастера в группы LuckPerms.
 *
 * <p>Мастер в LuckPerms не пишет: он отдаёт готовый {@code lp_groups}, а агент
 * приводит игрока к этому списку. Ключевое здесь — снимать лишнее, а не только
 * добавлять недостающее, иначе снятая на сайте роль остаётся в игре навсегда.
 */
public final class RoleSync implements RoleApplier {

    /**
     * {@code default} LuckPerms выдаёт всем неявно, и на нём висят базовые права.
     * Снести его как «лишнюю группу» — тихо отобрать у игроков всё сразу.
     */
    private static final Set<String> PROTECTED_GROUPS = Set.of("default");

    private final LuckPerms luckPerms;
    private final Logger log;

    public RoleSync(LuckPerms luckPerms, Logger log) {
        this.luckPerms = luckPerms;
        this.log = log;
    }

    @Override
    public CompletableFuture<Void> apply(UUID mcUuid, PlayerProfile profile) {
        List<CompletableFuture<Void>> prefixes = new ArrayList<>();
        for (RoleInfo role : profile.roles()) {
            prefixes.add(GroupPrefix.apply(luckPerms, role));
        }

        return CompletableFuture.allOf(prefixes.toArray(CompletableFuture[]::new))
                .thenCompose(ignored -> luckPerms.getUserManager()
                        .modifyUser(mcUuid, user -> reconcile(user, profile)))
                .exceptionally(error -> {
                    // Провалившаяся синхронизация не должна ронять вход: игрок
                    // войдёт с прежними группами, следующий вход попробует снова.
                    log.warn("Role sync failed for {}: {}", profile.username(), error.getMessage());
                    return null;
                });
    }

    /** Приводит набор групп игрока к списку от мастера. */
    private void reconcile(User user, PlayerProfile profile) {
        // Копия до изменений: ниже мы правим тот же набор узлов, по которому идём.
        List<InheritanceNode> managed = new ArrayList<>(user.getNodes(NodeType.INHERITANCE)).stream()
                .filter(RoleSync::isManaged)
                .filter(node -> !PROTECTED_GROUPS.contains(node.getGroupName()))
                .toList();

        Set<String> current = new HashSet<>();
        managed.forEach(node -> current.add(node.getGroupName()));
        Diff diff = diff(current, profile.lpGroups());

        // Снимаем по имени, а не по узлу: одна и та же группа может висеть
        // несколькими узлами, и уйти должны все.
        managed.stream()
                .filter(node -> diff.toRemove().contains(node.getGroupName()))
                .forEach(node -> user.data().remove(node));

        diff.toAdd().forEach(group -> user.data().add(InheritanceNode.builder(group).build()));
    }

    record Diff(Set<String> toRemove, Set<String> toAdd) {}

    /**
     * Чистое правило синхронизации, отделённое от API LuckPerms: только его и
     * можно проверить тестом, потому что узлы LuckPerms без запущенного плагина
     * не собираются.
     */
    static Diff diff(Set<String> current, Collection<String> target) {
        Set<String> toAdd = new HashSet<>(target);
        toAdd.removeAll(current);
        Set<String> toRemove = new HashSet<>(current);
        toRemove.removeAll(target);
        return new Diff(toRemove, toAdd);
    }

    /**
     * Агент распоряжается только постоянными группами в глобальном контексте.
     * Временная выдача (донат на месяц) и привязка к конкретному миру приходят
     * не от мастера, и снимать их он не вправе.
     */
    private static boolean isManaged(InheritanceNode node) {
        return !node.hasExpiry() && node.getContexts().isEmpty();
    }
}
