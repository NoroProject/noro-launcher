package dev.noro.client.player;

import java.util.List;

/**
 * Лаунчер → игрокский мод: свод правил и свои наказания.
 *
 * <p>Свой словарь кадров, а не общий со staff: ядро раздаёт кадры по имени, и
 * функция получает только те, что назвала своими. Панель разбора и свод правил
 * друг о друге не знают вовсе.
 */
public sealed interface PlayerFrames {

    /** Пункт свода. */
    record Rule(String id, String category_id, String code, String title, String description,
                int sort_order) {}

    /** Раздел свода; пункты без раздела идут в конце. */
    record Category(String id, String title, int sort_order) {}

    /** Вилка наказания по пункту: за что и насколько. */
    record Sanction(String rule_id, String kind, Long min_minutes, Long max_minutes) {}

    /** Наказание игрока — своё, а не чужое. */
    record Punishment(String id, String kind, String reason, String created_at,
                      String expires_at, String revoked_at, String rule_code) {

        /** Действует ли сейчас. Снятое и истёкшее остаётся историей. */
        public boolean active() {
            return revoked_at == null && (expires_at == null || expires_at.compareTo(nowIso()) > 0);
        }

        private static String nowIso() {
            return java.time.Instant.now().toString();
        }
    }

    /** Свод целиком: пункт и то, что за него бывает, читают вместе. */
    record Rules(List<Category> categories, List<Rule> rules, List<Sanction> sanctions)
            implements PlayerFrames {}

    record OwnPunishments(List<Punishment> punishments) implements PlayerFrames {}
}
