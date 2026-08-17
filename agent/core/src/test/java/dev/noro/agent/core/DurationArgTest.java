package dev.noro.agent.core;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

import org.junit.jupiter.api.Test;

/**
 * Единицы срока обязаны совпадать с тем, что мастер печатает в рамках правила
 * («up to 7d»): модератор читает подсказку и набирает её дословно.
 */
class DurationArgTest {

    @Test
    void understandsEveryUnitMasterPrints() {
        assertEquals(30L, DurationArg.parse("30m"));
        assertEquals(120L, DurationArg.parse("2h"));
        assertEquals(10_080L, DurationArg.parse("7d"));
        assertEquals(20_160L, DurationArg.parse("2w"));
        assertEquals(43_200L, DurationArg.parse("1mo"));
        assertEquals(525_600L, DurationArg.parse("1y"));
    }

    @Test
    void permanentHasItsOwnValue() {
        assertEquals(DurationArg.PERMANENT, DurationArg.parse("perm"));
        assertEquals(DurationArg.PERMANENT, DurationArg.parse("FOREVER"));
    }

    @Test
    void anythingElseIsNotATerm() {
        assertNull(DurationArg.parse("tomorrow"));
        assertNull(DurationArg.parse("7"));
        assertNull(DurationArg.parse("d"));
        assertNull(DurationArg.parse("0d"));
        assertNull(DurationArg.parse(""));
        assertNull(DurationArg.parse(null));
    }

    /** Обратный путь: минуты словами читает игрок на экране бана. */
    @Test
    void printsMinutesBack() {
        assertEquals("7d", DurationArg.format(10_080));
        assertEquals("1h 35m", DurationArg.format(95));
        assertEquals("forever", DurationArg.format(-1));
        // Ноль — это «меньше минуты»; «0m» на экране читается как «уже свободен».
        assertEquals("1m", DurationArg.format(0));
    }
}
