package dev.noro.client.staff.ui;

import dev.noro.client.staff.CaseIntents;
import dev.noro.client.staff.NoroStaff;
import dev.noro.client.ui.NoroButton;
import dev.noro.client.ui.Theme;
import java.util.function.Consumer;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

/**
 * Нижний ряд карточки: куда уйти и что решить.
 *
 * <p>Слева навигация и заметка, справа решения — наказание и закрытие. Порядок
 * не случайный: необратимое стоит дальше всего от кнопки «назад», в которую
 * целятся не глядя.
 */
final class CaseFooter {

    private final String caseId;
    private final Consumer<AbstractWidget> sink;
    private final Consumer<Screen> open;

    CaseFooter(String caseId, Consumer<AbstractWidget> sink, Consumer<Screen> open) {
        this.caseId = caseId;
        this.sink = sink;
        this.open = open;
    }

    void layout(int x, int y, int w, String rule, int tab) {
        // Каждая следующая кнопка начинается там, где кончилась прошлая: с
        // отступом числом они наезжали друг на друга, стоило подписи стать
        // длиннее на букву, — а длина у неё разная в каждом языке.
        int next = action(x, y, "noro.cases.action.queue", NoroStaff.PERM_VIEW,
                () -> open.accept(new QueueScreen()));
        action(next, y, "noro.cases.action.note", NoroStaff.PERM_VIEW,
                () -> open.accept(new NoteScreen(caseId)));

        Component resolve = Component.translatable("noro.cases.action.resolve");
        Component punish = Component.translatable("noro.cases.action.punish");
        int rw = NoroButton.widthFor(resolve);
        int pw = NoroButton.widthFor(punish);
        boolean allowed = NoroStaff.state().can(NoroStaff.PERM_RESOLVE);

        sink.accept(NoroButton.primary(x + w - rw, y, rw, resolve,
                        () -> open.accept(new ResolveScreen(caseId, rule)))
                .needs(allowed));
        sink.accept(NoroButton.of(x + w - rw - Theme.GRID - pw, y, pw, punish,
                        NoroButton.Variant.WARNING,
                        () -> open.accept(new PunishScreen(caseId, rule)))
                .needs(allowed));

        request(x + w / 2 - 12 * Theme.GRID, y, tab);
    }

    /**
     * Кнопка запроса — в той вкладке, где появится ответ.
     *
     * <p>«Спросить срез чата» на вкладке ленты просило то, чего на ней не видно:
     * результат приезжал в соседнюю. Теперь кнопка стоит там же, где его
     * показывают.
     */
    private void request(int x, int y, int tab) {
        switch (tab) {
            case 1 -> action(x, y, "noro.cases.action.chat", NoroStaff.PERM_CHAT,
                    () -> NoroStaff.cases().send(new CaseIntents.RequestChat(caseId)));
            case 2 -> action(x, y, "noro.cases.action.inventory", NoroStaff.PERM_INVENTORY,
                    () -> NoroStaff.cases().send(new CaseIntents.RequestInventory(caseId)));
            default -> { }
        }
    }

    /**
     * Кнопка рисуется всегда, но гаснет без права: прятать её значило бы врать
     * про то, чего у модератора нет. Решает всё равно мастер.
     */
    private int action(int x, int y, String key, String permission, Runnable onPress) {
        Component label = Component.translatable(key);
        int w = NoroButton.widthFor(label);
        sink.accept(NoroButton.ghost(x, y, w, label, onPress)
                .needs(NoroStaff.state().can(permission)));
        return x + w + Theme.GRID;
    }
}
