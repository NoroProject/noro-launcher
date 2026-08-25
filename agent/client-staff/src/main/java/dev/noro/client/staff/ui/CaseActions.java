package dev.noro.client.staff.ui;

import dev.noro.client.staff.Actions;
import dev.noro.client.staff.NoroStaff;
import dev.noro.client.staff.Screenshot;
import dev.noro.client.ui.Chip;
import dev.noro.client.ui.Theme;
import java.util.List;
import java.util.function.Consumer;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.network.chat.Component;

/**
 * Действия в мире внутри карточки: перемещения, осмотр, кадр.
 *
 * <p>Ровно те же, что кнопками в чате. Пока их там было больше, панель выглядела
 * урезанной версией чата, а не заменой ему: за телепортом приходилось выходить
 * из панели и искать строку меню в ленте сообщений.
 *
 * <p>Уходят они теми же командами {@code /case …}: серверный агент из-за панели
 * не пересобирается, а право проверяет мастер. Кнопка без права гаснет, но
 * остаётся на месте — прятать её значило бы врать про то, чего у модератора нет.
 */
final class CaseActions {

    /** Что за кнопкой: подпись, требуемое право и сама команда. */
    private record Item(String key, String permission, Consumer<String> run) {}

    private static final List<Item> ITEMS = List.of(
            new Item("noro.cases.action.tp.place", NoroStaff.PERM_VIEW, Actions::teleportPlace),
            new Item("noro.cases.action.tp.target", NoroStaff.PERM_VIEW, Actions::teleport),
            new Item("noro.cases.action.tp.reporter", NoroStaff.PERM_VIEW, Actions::teleportReporter),
            new Item("noro.cases.radial.back", NoroStaff.PERM_VIEW, Actions::back),
            new Item("noro.cases.radial.freeze", NoroStaff.PERM_RESOLVE, Actions::freeze),
            new Item("noro.cases.radial.watch", NoroStaff.PERM_WATCH, Actions::watch),
            new Item("noro.cases.radial.vanish", NoroStaff.PERM_VIEW, Actions::vanish),
            new Item("noro.cases.radial.invsee", NoroStaff.PERM_INVENTORY, Actions::invsee),
            new Item("noro.cases.radial.ender", NoroStaff.PERM_INVENTORY, Actions::enderChest),
            new Item("noro.cases.radial.shot", NoroStaff.PERM_VIEW, Screenshot::attach));

    private CaseActions() {}

    /**
     * Разложить кнопки рядами по ширине карточки.
     *
     * @return низ занятого места
     */
    static int layout(Consumer<AbstractWidget> sink, String caseId, int x, int y, int w) {
        int cursor = x;
        int line = y;
        for (Item item : ITEMS) {
            Component label = Component.translatable(item.key());
            int cw = Chip.widthFor(label);
            if (cursor > x && cursor + cw > x + w) {
                cursor = x;
                line += Chip.HEIGHT + Theme.GRID;
            }
            Chip chip = Chip.of(cursor, line, label, false, () -> {
                // Дело называем перед действием: у модератора может быть
                // несколько замков разом, а команды в мир номера не несут.
                Actions.use(caseId);
                item.run().accept(caseId);
            });
            chip.active = NoroStaff.state().can(item.permission());
            sink.accept(chip);
            cursor += cw + Theme.GRID;
        }
        return line + Chip.HEIGHT;
    }
}
