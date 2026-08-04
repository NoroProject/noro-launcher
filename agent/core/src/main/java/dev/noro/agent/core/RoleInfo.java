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

    /** Роль видна в игре, только если у неё есть и группа, и иконка. */
    public boolean hasPrefix() {
        return lpGroup != null && icon != null && !icon.isBlank();
    }
}
