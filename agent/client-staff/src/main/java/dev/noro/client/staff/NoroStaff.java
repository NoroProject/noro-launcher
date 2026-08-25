package dev.noro.client.staff;

import dev.noro.client.NoroCore;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.fml.common.Mod;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Инструменты модератора: панель разбора дел.
 *
 * <p>Отдельный мод от ядра, потому что раздаётся отдельно — правом
 * {@code noro.optional.<server>.<mod>}. Модератор видит его в списке модов,
 * остальные не видят ничего.
 *
 * <p>Прятать при этом нечего: в jar лежит рабочий код подсветки, и тот, кто его
 * декомпилирует, получит ESP без всяких прав. Это не аргумент против — готовых
 * ESP хватает и без нас, — но повод не тащить сюда ничего сверх подсветки
 * участников дела.
 */
@Mod(value = NoroStaff.ID, dist = Dist.CLIENT)
public final class NoroStaff {

    public static final String ID = "noro_staff";
    public static final Logger LOG = LoggerFactory.getLogger("NoroStaff");

    /** Права: по ним рисуются кнопки. Проверяет их всё равно мастер. */
    public static final String PERM_VIEW = "noro.mod.cases.view";
    public static final String PERM_CLAIM = "noro.mod.cases.claim";
    public static final String PERM_RESOLVE = "noro.mod.cases.resolve";
    public static final String PERM_CHAT = "noro.mod.cases.chat";
    public static final String PERM_INVENTORY = "noro.mod.cases.inventory";
    public static final String PERM_WATCH = "noro.mod.cases.watch";

    private static final Cases CASES = new Cases();

    public NoroStaff() {
        // Подключаемся к общему каналу: своего у staff нет и быть не должно —
        // соединение с лаунчером одно на все моды.
        NoroCore.register(CASES);
    }

    public static Cases cases() {
        return CASES;
    }

    public static CaseState state() {
        return CASES.state();
    }
}
