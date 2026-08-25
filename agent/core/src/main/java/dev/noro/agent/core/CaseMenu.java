package dev.noro.agent.core;

/**
 * Меню разбора: то, что модератор видит в чате, взяв дело.
 *
 * <p>Кнопки — ссылки вида {@code cmd:/case …}: платформа превращает их в клик,
 * выполняющий команду. Тем, у кого клик не сработал, команда всё равно видна в
 * подсказке при наведении — набрать её можно руками.
 */
final class CaseMenu {

    private CaseMenu() {}

    static String render(CaseSession session, String lang) {
        StringBuilder out = new StringBuilder();
        out.append(AgentStrings.get(lang, "case_menu_title", session.targetName())).append('\n');
        if (!session.reason().isBlank()) {
            out.append(AgentStrings.get(lang, "case_menu_reason", session.reason())).append('\n');
        }
        if (session.reporterName() != null) {
            out.append(AgentStrings.get(lang, "case_menu_reporter", session.reporterName())).append('\n');
        }

        // Цвет — по роду занятия, а не для красоты: три ряда кнопок подряд
        // сливаются в стену, и «закрыть» оказывается в ней таким же на вид, как
        // «телепорт». Синий — перемещения, кремовый — осмотр, красный —
        // необратимое.
        button(out, lang, "case_menu_tp_place", "/case tp place", MOVE);
        button(out, lang, "case_menu_tp_target", "/case tp target", MOVE);
        if (session.reporterName() != null) {
            button(out, lang, "case_menu_tp_reporter", "/case tp reporter", MOVE);
        }
        button(out, lang, "case_menu_back", "/case back", MOVE);
        out.append('\n');

        button(out, lang, "case_menu_chat", "/case chat", LOOK);
        button(out, lang, "case_menu_inventory", "/case inv", LOOK);
        button(out, lang, session.watching() ? "case_menu_unwatch" : "case_menu_watch", "/case watch", LOOK);
        button(out, lang, "case_menu_freeze", "/case freeze", LOOK);
        out.append('\n');

        button(out, lang, "case_menu_close", "/case close", END);
        button(out, lang, "case_menu_release", "/case release", DIM);
        return out.toString();
    }

    /** Перемещения, осмотр, необратимое и просто «вернуть как было». */
    private static final String MOVE = "<#7fb2ff>";
    private static final String LOOK = "<#f3e7b3>";
    private static final String END = "<#ff6b8b>";
    private static final String DIM = "<#b8c4e0>";

    /**
     * Кнопка вида {@code [чат]}.
     *
     * <p>Скобки нарисованы, а не подразумеваются: без них ряд коротких слов
     * читается как одна фраза, и непонятно, где кончается одна кнопка и
     * начинается другая. Кликается вся кнопка вместе со скобками — попасть в
     * слово из трёх букв мышью тяжелее, чем кажется.
     */
    private static void button(StringBuilder out, String lang, String key, String command, String color) {
        out.append(" [").append(DIM).append('[').append(color)
                .append(AgentStrings.get(lang, key))
                .append(DIM).append("]](cmd:")
                .append(command)
                .append(')');
    }
}
