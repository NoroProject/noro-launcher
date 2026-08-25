package dev.noro.client.staff;

import dev.noro.client.ui.Radial;
import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.player.Player;

/**
 * Колесо разбора поверх общего {@link Radial} из ядра.
 *
 * <p>Само колесо — механика ядра; здесь только пункты и то, что за ними стоит.
 * Действия в мире уходят командами {@code /case …}: серверный агент из-за
 * панели не пересобирается, а право проверяет мастер.
 */
public final class CaseWheel {

    private static final Radial WHEEL = Radial.of()
            .item("noro.cases.radial.tp", () -> withCase(CaseWheel::teleport))
            .item("noro.cases.radial.back", () -> withCase(Actions::back))
            .item("noro.cases.radial.vanish", () -> withCase(Actions::vanish))
            .item("noro.cases.radial.freeze", () -> withCase(Actions::freeze))
            .item("noro.cases.radial.watch", () -> withCase(CaseWheel::watch))
            .item("noro.cases.radial.invsee", () -> withCase(Actions::invsee))
            .item("noro.cases.radial.ender", () -> withCase(Actions::enderChest))
            .item("noro.cases.radial.shot", () -> withCase(Screenshot::attach));

    private CaseWheel() {}

    public static Radial wheel() {
        return WHEEL;
    }

    /** Колесо открывается, только когда есть что разбирать. */
    public static void hold(boolean down) {
        WHEEL.hold(down, NoroStaff.state().open() != null);
    }

    private static void withCase(java.util.function.Consumer<String> action) {
        CaseModels.View view = NoroStaff.state().open();
        if (view != null && view.brief() != null) {
            // Как и в карточке: агент должен знать, о каком деле речь, — команды
            // в мир номера не несут, а замков у модератора может быть несколько.
            Actions.use(view.brief().id());
            action.accept(view.brief().id());
        }
    }

    /**
     * Телепорт снимает кадр сам: контекст в деле появляется без напоминаний, а
     * вспоминать про скриншот посреди разбора поздно.
     */
    private static void teleport(String caseId) {
        Actions.teleport(caseId);
        Screenshot.note("teleport");
        Screenshot.attach(caseId);
    }

    private static void watch(String caseId) {
        Actions.watch(caseId);
        Screenshot.note("watch");
        Screenshot.attach(caseId);
        // Числа считаются по цели, которую клиент уже видит через трекинг:
        // новых позиций мод не запрашивает.
        Telemetry.watch(target());
    }

    private static Player target() {
        CaseModels.View view = NoroStaff.state().open();
        Minecraft mc = Minecraft.getInstance();
        if (view == null || view.brief() == null || mc.level == null) {
            return null;
        }
        String name = view.brief().target_name();
        if (name == null) {
            return null;
        }
        for (Player p : mc.level.players()) {
            if (name.equals(p.getName().getString())) {
                return p;
            }
        }
        return null;
    }
}
