package dev.noro.agent.mod;

import com.mojang.authlib.GameProfile;
import java.util.UUID;

/**
 * UUID из профиля, с которым игрок логинится.
 *
 * <p>Отдельный файл ради одной строки: в 1.21.9 {@code GameProfile} стал record,
 * и {@code getId()} там больше нет. Разрыв закрыт в одном месте, а не по разу
 * в каждой из двух точек входа, — иначе его пришлось бы чинить дважды.
 */
final class ProfileId {

    private ProfileId() {}

    static UUID of(GameProfile profile) {
        //#if MC>=12109
        return profile.id();
        //#else
        //$$ return profile.getId();
        //#endif
    }
}
