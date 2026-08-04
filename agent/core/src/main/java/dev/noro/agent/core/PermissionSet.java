package dev.noro.agent.core;

import java.util.Collection;
import java.util.Set;

/**
 * Права игрока, как их прислал мастер, с проверкой по wildcard.
 *
 * <p>Семантика повторяет {@code permission_matches} из
 * {@code crates/schema/src/permissions.rs} буква в букву. Разойдись они — сайт
 * и игра начнут отвечать по-разному на один и тот же вопрос, а заметить это
 * можно будет только вручную и не сразу.
 *
 * <p>Набор неизменяемый: его читают из игрового потока на каждую проверку прав,
 * а пишут один раз на логине.
 */
public final class PermissionSet {

    private static final PermissionSet EMPTY = new PermissionSet(Set.of());

    /** Право «всё сразу»: проверяется первым, потому что перебор ему не нужен. */
    private static final String ALL = "*";

    private final Set<String> patterns;

    private PermissionSet(Set<String> patterns) {
        this.patterns = patterns;
    }

    public static PermissionSet of(Collection<String> permissions) {
        if (permissions == null || permissions.isEmpty()) {
            return EMPTY;
        }
        return new PermissionSet(Set.copyOf(permissions));
    }

    public static PermissionSet empty() {
        return EMPTY;
    }

    /**
     * Узлы в исходном виде, вместе с wildcard'ами и префиксами.
     *
     * <p>Нужны там, где право не спрашивают, а выдают: Bukkit хранит у игрока
     * список имён, и раскрывать шаблоны приходится заранее.
     */
    public Set<String> patterns() {
        return patterns;
    }

    public boolean has(String required) {
        if (required == null || patterns.isEmpty()) {
            return false;
        }
        // Точное совпадение и суперадмин — самые частые случаи, и оба ловятся
        // хешем, без обхода всего набора.
        if (patterns.contains(required) || patterns.contains(ALL)) {
            return true;
        }
        for (String pattern : patterns) {
            if (matches(pattern, required)) {
                return true;
            }
        }
        return false;
    }

    /**
     * {@code *} подходит ко всему, {@code foo.*} — к {@code foo} и к
     * {@code foo.<что-угодно>}, остальное должно совпасть точно.
     */
    public static boolean matches(String pattern, String target) {
        if (ALL.equals(pattern) || pattern.equals(target)) {
            return true;
        }
        if (!pattern.endsWith(".*")) {
            return false;
        }
        String prefix = pattern.substring(0, pattern.length() - 2);
        return target.equals(prefix) || target.startsWith(prefix + ".");
    }
}
