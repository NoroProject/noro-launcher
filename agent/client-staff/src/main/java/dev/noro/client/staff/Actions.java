package dev.noro.client.staff;

import net.minecraft.client.Minecraft;
import net.minecraft.client.player.LocalPlayer;

/**
 * Действия в мире — теми же командами {@code /case …}, что и кнопки в чате.
 *
 * <p>Дело в команде не называется: у агента одна сессия разбора на модератора,
 * и она уже знает, какое дело открыто. Передавать сюда id значило бы придумать
 * агенту грамматику, которой у него нет, — что и случилось: {@code /case tp
 * <uuid>} он читал как «телепорт к цели по имени uuid» и не находил её.
 *
 * <p>Серверный агент из-за панели не трогается вообще: ни одна из восьмидесяти
 * сборок не пересобирается. Право проверяет мастер, как и раньше, а панель не
 * даёт возможностей, которых нет из чата, — иначе «опциональный» мод перестал
 * бы быть правдой, и модератор без него оказался бы второго сорта.
 */
public final class Actions {

    private Actions() {}

    /** Дело, на которое агент уже переключён: второй раз просить незачем. */
    private static String active;

    /**
     * Сказать агенту, какое дело сейчас на экране.
     *
     * <p>У модератора может быть несколько замков разом, а команды в мир номера
     * дела не несут. Без этого «телепорт» уходил бы в то дело, которое агент
     * запомнил последним, — не обязательно в открытое.
     *
     * <p>Только при смене дела: на каждое переключение агент печатает меню в
     * чат, и звать его перед каждой кнопкой значило бы забить ленту меню.
     */
    public static void use(String caseId) {
        if (caseId != null && !caseId.equals(active)) {
            active = caseId;
            run("case use " + caseId);
        }
    }

    /** Забыть переключение: после отпускания дела агент о нём тоже не помнит. */
    public static void forget() {
        active = null;
    }

    public static void teleport(String caseId) {
        run("case tp target");
    }

    /** К месту жалобы — туда, где это произошло, а не где цель сейчас. */
    public static void teleportPlace(String caseId) {
        run("case tp place");
    }

    public static void teleportReporter(String caseId) {
        run("case tp reporter");
    }

    public static void back(String caseId) {
        run("case tp back");
    }

    public static void freeze(String caseId) {
        run("case freeze");
    }

    /** Ваниш — отдельная команда агента, не подкоманда разбора. */
    public static void vanish(String caseId) {
        run("vanish");
    }

    /** Слежка переключается тем же вызовом: агент сам знает, идёт она или нет. */
    public static void watch(String caseId) {
        run("case watch");
    }

    public static void inventory(String caseId) {
        run("case inv");
    }

    /** Инвентарь цели контейнером: из него можно забрать улику. */
    public static void invsee(String caseId) {
        run("case invsee");
    }

    public static void enderChest(String caseId) {
        run("case ender");
    }

    private static void run(String command) {
        LocalPlayer player = Minecraft.getInstance().player;
        if (player == null) {
            return;
        }
        player.connection.sendCommand(command);
    }
}
