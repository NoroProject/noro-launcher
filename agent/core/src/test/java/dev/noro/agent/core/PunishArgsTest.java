package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

import org.junit.jupiter.api.Test;

/**
 * Разбор команды наказания. Проверяется то, из-за чего модератор в игре получает
 * не тот бан, который хотел: срок, спутанный с причиной, и наоборот.
 */
class PunishArgsTest {

    private static PunishArgs parse(String line, boolean timed) {
        return PunishArgs.parse(line.split(" "), timed);
    }

    @Test
    void takesTermRuleAndReason() {
        PunishArgs args = parse("Steve 7d @chat.spam mass advertising", true);

        assertEquals("Steve", args.target());
        assertEquals(10_080L, args.minutes());
        assertEquals("chat.spam", args.ruleCode());
        assertEquals("mass advertising", args.reason());
    }

    /** Срок и правило — в любом порядке: модератор печатает как удобнее. */
    @Test
    void termAndRuleComeInAnyOrder() {
        PunishArgs first = parse("Steve 1.1 30m spam", true);
        assertEquals("1.1", first.ruleCode());
        assertEquals(30L, first.minutes());

        PunishArgs second = parse("Steve 30m 1.1 spam", true);
        assertEquals("1.1", second.ruleCode());
        assertEquals(30L, second.minutes());
    }

    /** Код пункта пишется и без собачки: точка отличает его от слова причины. */
    @Test
    void readsRuleCodeWithoutAtSign() {
        assertEquals("1.1", parse("Steve 1.1 оскорбления", true).ruleCode());
        assertNull(parse("Steve оскорбления игрока", true).ruleCode());
    }

    /** Срок необязателен: без него наказание вечное, и это решает мастер. */
    @Test
    void missingTermMeansForever() {
        PunishArgs args = parse("Steve @chat.spam being rude", true);

        assertNull(args.minutes());
        assertEquals("chat.spam", args.ruleCode());
        assertEquals("being rude", args.reason());
    }

    @Test
    void permanentIsTheSameAsNoTerm() {
        assertNull(parse("Steve perm cheating", true).minutes());
    }

    /**
     * Слово, похожее на срок, но не срок, остаётся причиной: «3 warnings» не
     * должно превращаться в трёхминутный бан.
     */
    @Test
    void reasonThatLooksLikeTermStaysReason() {
        PunishArgs args = parse("Steve 3 warnings already", true);

        assertNull(args.minutes());
        assertEquals("3 warnings already", args.reason());
    }

    /** У предупреждения срока нет — первое слово после ника уже причина. */
    @Test
    void warnKeepsTermInReason() {
        PunishArgs args = parse("Steve 7d of nonsense", false);

        assertNull(args.minutes());
        assertEquals("7d of nonsense", args.reason());
    }

    /**
     * Назвал пункт и ничего не написал — причина пустая, её достроит вызывающий
     * из формулировки правила. Это самый частый способ наказать в игре.
     */
    @Test
    void ruleWithoutReasonIsAllowed() {
        PunishArgs args = parse("Steve 30m 1.1", true);

        assertEquals("1.1", args.ruleCode());
        assertEquals(30L, args.minutes());
        assertEquals("", args.reason());
    }

    @Test
    void refusesNothingToGoOn() {
        assertNull(parse("Steve", true));
        // Ни правила, ни внятной причины: мастер такое всё равно не примет.
        assertNull(parse("Steve 7d", true));
        assertNull(parse("Steve 7d ok", true));
    }
}
