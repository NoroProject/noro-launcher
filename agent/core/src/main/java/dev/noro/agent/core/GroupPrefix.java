package dev.noro.agent.core;

import java.util.Collection;
import java.util.concurrent.CompletableFuture;
import net.luckperms.api.LuckPerms;
import net.luckperms.api.model.group.Group;
import net.luckperms.api.node.NodeType;
import net.luckperms.api.node.types.ChatMetaNode;
import net.luckperms.api.node.types.PrefixNode;
import net.luckperms.api.node.types.SuffixNode;

/**
 * Префикс и суффикс группы LuckPerms — проекция оформления роли.
 *
 * <p>Вес равен {@code sort_order} роли, поэтому у игрока с несколькими ролями
 * в чате окажется префикс старшей — тот же порядок, что и на сайте.
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
        if (role.lpGroup() == null || !role.hasDecoration()) {
            return CompletableFuture.completedFuture(null);
        }
        String prefix = role.prefixText();
        String suffix = role.suffixText();

        return luckPerms.getGroupManager()
                .createAndLoadGroup(role.lpGroup())
                .thenCompose(group -> {
                    if (matches(group, prefix, suffix, role.sortOrder())) {
                        return CompletableFuture.completedFuture(null);
                    }
                    // Группа — проекция роли, поэтому её меты мы держим целиком:
                    // иначе смена префикса на сайте оставила бы старый рядом с
                    // новым, и в чате оказалась бы пара подписей.
                    group.data().clear(NodeType.PREFIX::matches);
                    group.data().clear(NodeType.SUFFIX::matches);
                    if (!prefix.isEmpty()) {
                        group.data().add(PrefixNode.builder(prefix, role.sortOrder()).build());
                    }
                    if (!suffix.isEmpty()) {
                        group.data().add(SuffixNode.builder(suffix, role.sortOrder()).build());
                    }
                    return luckPerms.getGroupManager().saveGroup(group);
                });
    }

    /** Сверка до записи: apply зовётся на каждый вход, а группа меняется редко. */
    private static boolean matches(Group group, String prefix, String suffix, int priority) {
        return single(group.getNodes(NodeType.PREFIX), prefix, priority)
                && single(group.getNodes(NodeType.SUFFIX), suffix, priority);
    }

    /**
     * Ровно одна мета с нужным значением — либо ни одной, если значения нет.
     * Всё прочее считаем расхождением и переписываем.
     */
    private static boolean single(Collection<? extends ChatMetaNode<?, ?>> nodes, String value, int priority) {
        if (value.isEmpty()) {
            return nodes.isEmpty();
        }
        if (nodes.size() != 1) {
            return false;
        }
        ChatMetaNode<?, ?> node = nodes.iterator().next();
        return node.getPriority() == priority && value.equals(node.getMetaValue());
    }
}
