package dev.noro.agent.core;

import java.util.UUID;

/**
 * Разбор, который модератор ведёт прямо сейчас.
 *
 * <p>Позиция запоминается на входе в режим: модератор телепортируется по делу
 * несколько раз, и «вернуться» должно возвращать туда, где он был до разбора,
 * а не туда, откуда прыгнул в последний раз.
 *
 * @param origin где модератор стоял до входа; {@code null} — платформа не
 *               умеет отдавать позицию, кнопка «назад» тогда просто откажет
 */
public record CaseSession(
        UUID caseId,
        UUID target,
        String targetName,
        UUID reporter,
        String reporterName,
        String reason,
        String world,
        Double x,
        Double y,
        Double z,
        GameBridge.Position origin,
        boolean watching) {

    public boolean hasPlace() {
        return world != null && x != null && y != null && z != null;
    }

    public CaseSession watching(boolean value) {
        return new CaseSession(
                caseId, target, targetName, reporter, reporterName, reason, world, x, y, z, origin, value);
    }
}
