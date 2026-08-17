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
        /** Символ-иконка либо {@code null}. Иконка — не префикс: это один глиф
         * рядом с ником, тогда как префикс произвольной длины. */
        String icon,
        /** Строка перед ником, с цветами через {@code &}. {@code null} — не задана. */
        String prefix,
        /** Строка после ника. Тот же формат. */
        String suffix,
        /** Больше — важнее. Совпадает с весом префикса в LuckPerms. */
        int sortOrder) {

    /**
     * Что показывать перед ником: заданный префикс, иначе иконка в цвете роли.
     *
     * <p>Запасной вариант — ради сборок, живших до появления поля: там
     * префиксом работала иконка, и молча снять её обновлением значило бы
     * стереть оформление чата на работающем сервере.
     *
     * @return готовая legacy-строка либо пустая, если показывать нечего
     */
    public String prefixText() {
        if (prefix != null && !prefix.isBlank()) {
            return PrefixFormat.legacy(prefix);
        }
        String fromIcon = PrefixFormat.of(color, icon);
        return fromIcon == null ? "" : fromIcon;
    }

    /** Строка после ника. Пустая, если суффикс не задан. */
    public String suffixText() {
        return suffix == null || suffix.isBlank() ? "" : PrefixFormat.legacy(suffix);
    }

    /** Есть ли у роли что показать вокруг ника. */
    public boolean hasDecoration() {
        return !prefixText().isEmpty() || !suffixText().isEmpty();
    }

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
