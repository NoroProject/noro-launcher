package dev.noro.agent.core;

import java.util.Comparator;
import java.util.List;
import java.util.Locale;
import java.util.stream.Collectors;
import java.util.stream.Stream;

/**
 * Профиль мастера, разложенный по ключам плейсхолдеров.
 *
 * <p>Логика одна на все платформы: PlaceholderAPI на Paper и Text Placeholder
 * API на Fabric различаются только тем, как ключ доезжает сюда и во что
 * заворачивается ответ. Разведи это по реализациям — и `%noro_role%` начнёт
 * означать разное на разных ядрах.
 */
public final class PlaceholderValues {

    /** Ключи без хвоста: платформы регистрируют их поимённо. */
    public static final List<String> KEYS = List.of(
            "username", "uuid", "banned", "allowed", "muted", "mute_reason", "mute_notice",
            "prefix", "prefix_plain", "suffix", "suffix_plain",
            "role", "role_name", "role_color", "role_color_legacy", "role_icon", "role_sort",
            "role_prefix", "role_suffix",
            "roles", "roles_icons", "role_count",
            "group", "groups", "group_count",
            "skin_url", "cape_url", "permission_count");

    /** Ключи с хвостом: {@code has_role_admin}, {@code permission_noro.fly}. */
    public static final List<String> PREFIXED_KEYS = List.of("has_role", "has_group", "permission");

    /**
     * Всё, что стоит регистрировать поимённо там, где плейсхолдеры объявляются
     * заранее. Ключи с хвостом получают его отдельным аргументом, поэтому
     * регистрируются так же, как остальные: {@code %noro:has_role admin%}.
     */
    public static final List<String> ALL_KEYS =
            Stream.concat(KEYS.stream(), PREFIXED_KEYS.stream()).toList();

    private static final String SEPARATOR = ", ";

    private PlaceholderValues() {}

    /**
     * @return значение либо {@code null}, если ключ не наш — на таком ответе
     *         платформа обязана оставить текст плейсхолдера нетронутым, чтобы
     *         его разобрал тот, кому он и адресован
     */
    public static String resolve(PlayerProfile profile, String key) {
        if (profile == null || key == null) {
            return null;
        }
        String name = key.toLowerCase(Locale.ROOT);
        // Строго в этом порядке: `permission_count` — имя из списка ниже, а не
        // вопрос про право с узлом `count`, и разбор с хвостом его бы перехватил.
        String fixed = fixed(profile, name);
        return fixed != null ? fixed : prefixed(profile, name);
    }

    private static String fixed(PlayerProfile profile, String name) {
        // Роль для показа и роль для префикса — разные: у старшей роли может не
        // быть оформления, и тогда в игре виден префикс следующей за ней. То же
        // правило, по которому LuckPerms выбирает префикс по весам.
        RoleInfo top = top(profile, false);
        RoleInfo shown = top(profile, true);
        return switch (name) {
            case "username" -> text(profile.username());
            case "uuid" -> String.valueOf(profile.uuid());
            case "banned" -> String.valueOf(profile.banned());
            case "allowed" -> String.valueOf(profile.allowed());
            case "muted" -> String.valueOf(profile.activeMute() != null);
            case "mute_reason" -> profile.activeMute() == null ? "" : text(profile.activeMute().reason());
            case "mute_notice" -> {
                PunishmentInfo mute = profile.activeMute();
                yield mute == null ? null : MessageRender.render(MessageTemplates.defaults().screen(mute), mute, profile.username());
            }
            case "skin_url" -> text(profile.skinUrl());
            case "cape_url" -> text(profile.capeUrl());
            case "prefix" -> shown == null ? "" : shown.prefixText();
            // Без цветовых кодов: нужно там, где строку кладут в поле, которое
            // само не умеет legacy, — заголовок скорборда, лог, веб-виджет.
            case "prefix_plain" -> shown == null ? "" : PrefixFormat.plain(shown.prefixText());
            case "suffix" -> shown == null ? "" : shown.suffixText();
            case "suffix_plain" -> shown == null ? "" : PrefixFormat.plain(shown.suffixText());
            case "role_prefix" -> top == null ? "" : top.prefixText();
            case "role_suffix" -> top == null ? "" : top.suffixText();
            case "role" -> top == null ? "" : text(top.displayName());
            case "role_name" -> top == null ? "" : text(top.name());
            case "role_color" -> top == null ? "" : text(top.color());
            case "role_color_legacy" -> top == null ? "" : PrefixFormat.color(top.color());
            case "role_icon" -> top == null ? "" : text(top.icon());
            case "role_sort" -> top == null ? "" : String.valueOf(top.sortOrder());
            case "role_count" -> String.valueOf(profile.roles().size());
            case "roles" -> byImportance(profile).map(role -> text(role.displayName()))
                    .collect(Collectors.joining(SEPARATOR));
            // Слитно и без разделителя: это готовая строка для таба, где иконки
            // всех ролей стоят подряд, каждая в своём цвете.
            case "roles_icons" -> byImportance(profile).filter(RoleInfo::hasPrefix)
                    .map(role -> PrefixFormat.of(role.color(), role.icon()))
                    .collect(Collectors.joining());
            case "group" -> profile.lpGroups().isEmpty() ? "" : profile.lpGroups().get(0);
            case "groups" -> String.join(SEPARATOR, profile.lpGroups());
            case "group_count" -> String.valueOf(profile.lpGroups().size());
            case "permission_count" -> String.valueOf(profile.permissions().size());
            default -> null;
        };
    }

    /** {@code has_role_<имя>}, {@code has_group_<имя>}, {@code permission_<узел>}. */
    private static String prefixed(PlayerProfile profile, String key) {
        String role = tail(key, "has_role_");
        if (role != null) {
            return String.valueOf(profile.roles().stream().anyMatch(it -> role.equalsIgnoreCase(it.name())));
        }
        String group = tail(key, "has_group_");
        if (group != null) {
            return String.valueOf(profile.lpGroups().stream().anyMatch(group::equalsIgnoreCase));
        }
        String node = tail(key, "permission_");
        if (node != null) {
            // Через matches, а не PermissionSet.of: набор пришлось бы копировать
            // на каждый запрос, а запросов у плейсхолдера столько же, сколько
            // перерисовок таба.
            return String.valueOf(profile.permissions().stream().anyMatch(it -> PermissionSet.matches(it, node)));
        }
        return null;
    }

    private static String tail(String key, String prefix) {
        if (!key.startsWith(prefix) || key.length() == prefix.length()) {
            return null;
        }
        return key.substring(prefix.length());
    }

    /**
     * Package-private: наружу флаг не выставляем — там на два случая есть два
     * имени, {@code NoroAgentApi.prefixRole} и {@code topRole}.
     */
    static RoleInfo top(PlayerProfile profile, boolean decoratedOnly) {
        if (profile == null) {
            return null;
        }
        return profile.roles().stream()
                .filter(role -> !decoratedOnly || role.hasDecoration())
                .max(Comparator.comparingInt(RoleInfo::sortOrder))
                .orElse(null);
    }

    private static Stream<RoleInfo> byImportance(PlayerProfile profile) {
        return profile.roles().stream()
                .sorted(Comparator.comparingInt(RoleInfo::sortOrder).reversed());
    }

    /**
     * Пустая строка вместо {@code null}: плейсхолдер без значения должен
     * исчезать из строки, а не превращаться в «null» посреди ника.
     */
    private static String text(String value) {
        return value == null ? "" : value;
    }
}
