package dev.noro.agent.core;

/**
 * Роль игрока в том виде, в каком её отдаёт мастер.
 *
 * <p>Поля приходят в snake_case; преобразование делает единая политика имён
 * Gson в {@link MasterClient}, поэтому аннотации на каждое поле не нужны.
 */
public record RoleInfo(
        String name,
        String displayName,
        /** Группа LuckPerms. {@code null} — роль в игру не проецируется. */
        String lpGroup,
        /** {@code #rrggbb} либо {@code null}. */
        String color,
        /** Символ-иконка либо {@code null} — тогда роль не даёт префикса. */
        String icon,
        /** Больше — важнее. Совпадает с весом префикса в LuckPerms. */
        int sortOrder) {

    /**
     * Даёт ли роль префикс — то есть есть ли у неё иконка.
     *
     * <p>Группа LuckPerms здесь не при чём, хотя раньше требовалась и она.
     * Из-за этого на сервере без LuckPerms — а без него живут все, кому он
     * ломает запуск, — префикса не было вообще ни у кого: роль с иконкой, но
     * без `lp_group` не считалась дающей префикс, и `NoroAgentApi.value(…,
     * "prefix")` отвечал пустой строкой. Группа нужна ровно одному потребителю,
     * {@link GroupPrefix}, и спрашивает он про неё сам.
     */
    public boolean hasPrefix() {
        return icon != null && !icon.isBlank();
    }
}
