package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

import org.junit.jupiter.api.Test;

/**
 * Разбор команды наказания. Проверяется то, из-за чего модератор в игре
 * получает не тот бан, который хотел: срок, спутанный с причиной, и наоборот.
 */
class PunishArgsTest {

    @Test
    void takesTermRuleAndReason() {
        PunishArgs args = PunishArgs.parse("Steve 7d @chat.spam mass advertising".split(" "), true);

        assertEquals("Steve", args.target());
        assertEquals(10_080L, args.minutes());
        assertEquals("chat.spam", args.ruleCode());
        assertEquals("mass advertising", args.reason());
    }

    /** Срок необязателен: без него наказание вечное, и это решает мастер. */
    @Test
    void missingTermMeansForever() {
        PunishArgs args = PunishArgs.parse("Steve @chat.spam being rude".split(" "), true);

        assertNull(args.minutes());
        assertEquals("chat.spam", args.ruleCode());
        assertEquals("being rude", args.reason());
    }

    @Test
    void permanentIsTheSameAsNoTerm() {
        assertNull(PunishArgs.parse("Steve perm cheating".split(" "), true).minutes());
    }

    /**
     * Слово, похожее на срок, но не срок, остаётся причиной: «3 warnings» не
     * должно превращаться в трёхминутный бан.
     */
    @Test
    void reasonThatLooksLikeTermStaysReason() {
        PunishArgs args = PunishArgs.parse("Steve 3 warnings already".split(" "), true);

        assertNull(args.minutes());
        assertEquals("3 warnings already", args.reason());
    }

    /** У предупреждения срока нет — первое слово после ника уже причина. */
    @Test
    void warnKeepsTermInReason() {
        PunishArgs args = PunishArgs.parse("Steve 7d of nonsense".split(" "), false);

        assertNull(args.minutes());
        assertEquals("7d of nonsense", args.reason());
    }

    @Test
    void refusesEmptyOrTooShortReason() {
        assertNull(PunishArgs.parse("Steve".split(" "), true));
        assertNull(PunishArgs.parse("Steve 7d".split(" "), true));
        assertNull(PunishArgs.parse("Steve 7d ok".split(" "), true));
    }
}
