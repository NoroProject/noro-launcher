package dev.noro.agent.core;

import java.util.Collection;
import java.util.concurrent.CompletableFuture;
import net.luckperms.api.LuckPerms;
import net.luckperms.api.model.group.Group;
import net.luckperms.api.node.NodeType;
import net.luckperms.api.node.types.PrefixNode;

/**
 * Префикс группы LuckPerms — проекция иконки и цвета роли.
 *
 * <p>Вес префикса равен {@code sort_order} роли, поэтому у игрока с несколькими
 * ролями в чате окажется иконка старшей — тот же порядок, что и на сайте.
 */
public final class GroupPrefix {

    private GroupPrefix() {}

    /**
     * Приводит префикс группы к текущему виду роли, создавая группу, если её
     * ещё нет: на свежем сервере иначе не появилось бы вообще ничего.
     */
    public static CompletableFuture<Void> apply(LuckPerms luckPerms, RoleInfo role) {
        // Группу спрашиваем здесь: префикс сам по себе от LuckPerms не зависит,
        // а вот проецировать роль без `lp_group` попросту некуда.
        if (role.lpGroup() == null || !role.hasPrefix()) {
            return CompletableFuture.completedFuture(null);
        }
        String desired = PrefixFormat.of(role.color(), role.icon());

        return luckPerms.getGroupManager()
                .createAndLoadGroup(role.lpGroup())
                .thenCompose(group -> {
                    if (matches(group, desired, role.sortOrder())) {
                        return CompletableFuture.completedFuture(null);
                    }
                    // Группа — проекция роли, поэтому её префикс мы держим целиком:
                    // иначе смена цвета на сайте оставила бы старый префикс рядом
                    // с новым, и в чате осталась бы пара иконок.
                    group.data().clear(NodeType.PREFIX::matches);
                    group.data().add(PrefixNode.builder(desired, role.sortOrder()).build());
                    return luckPerms.getGroupManager().saveGroup(group);
                });
    }

    /** Сверка до записи: apply зовётся на каждый вход, а группа меняется редко. */
    private static boolean matches(Group group, String desired, int priority) {
        Collection<PrefixNode> nodes = group.getNodes(NodeType.PREFIX);
        if (nodes.size() != 1) {
            return false;
        }
        PrefixNode node = nodes.iterator().next();
        return node.getPriority() == priority && desired.equals(node.getMetaValue());
    }
}
