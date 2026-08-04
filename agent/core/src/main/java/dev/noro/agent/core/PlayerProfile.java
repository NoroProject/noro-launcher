package dev.noro.agent.core;

import java.util.List;
import java.util.UUID;

/**
 * Ответ {@code GET /api/agent/players/{mc_uuid}}.
 *
 * <p>{@link #allowed} считает мастер: туда уже входят бан и право на вход в
 * ограниченную сборку. Агент это не пересчитывает — иначе три реализации
 * разъедутся между собой и с сайтом.
 */
public record PlayerProfile(
        UUID uuid,
        String username,
        boolean banned,
        boolean allowed,
        List<RoleInfo> roles,
        /** Есть всегда: свой скин игрока либо общий Стив. */
        String skinUrl,
        String capeUrl,
        /** Группы LuckPerms в порядке важности — готовый результат от мастера. */
        List<String> lpGroups,
        /**
         * Права на этом сервере: свои и от ролей. Сюда же мастер кладёт узлы
         * {@code prefix.<вес>.<значение>} — LuckPerms хранит префикс так же,
         * и моды, читающие меты из прав, ищут именно там.
         */
        List<String> permissions) {

    public PlayerProfile {
        roles = roles == null ? List.of() : List.copyOf(roles);
        lpGroups = lpGroups == null ? List.of() : List.copyOf(lpGroups);
        permissions = permissions == null ? List.of() : List.copyOf(permissions);
    }
}
