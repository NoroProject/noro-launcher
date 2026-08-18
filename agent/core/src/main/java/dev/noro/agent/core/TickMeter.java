package dev.noro.agent.core;

/**
 * Свой счётчик тиков.
 *
 * <p>Не через {@code Bukkit.getTPS()} и не через {@code getAverageTickTime()}:
 * на диапазоне 1.18.2 → 26.x эти API разъезжаются, и каждая версия означала бы
 * ещё одну директиву препроцессора. Хук конца тика есть везде — на нём и
 * считаем.
 *
 * <p>Побочная выгода важнее основной. {@link #onTickEnd()} зовут из игрового
 * потока, а {@link #stalledSeconds()} читает фоновый демон. Если игровой поток
 * встал, счётчик замирает — а сообщить об этом всё ещё есть кому. Это и есть
 * детектор зависания: TPS у повисшего сервера просто перестаёт обновляться, и
 * по нему одному отличить «завис» от «тихо» нельзя.
 *
 * <p>Писать сюда может только игровой поток, читать — любой. Поэтому итоги окна
 * не считаются на чтении, а публикуются писателем в {@code volatile}-поля: так
 * обходимся без единой блокировки на горячем пути.
 */
public final class TickMeter {

    /** Итог подводится раз в столько наносекунд — пять секунд. */
    private static final long WINDOW_NANOS = 5_000_000_000L;

    private static final double NANOS_PER_SECOND = 1_000_000_000.0;
    private static final double NANOS_PER_MILLI = 1_000_000.0;

    // Только для игрового потока.
    private long windowStart;
    private int windowTicks;
    private long windowWorkNanos;
    private long tickStart;
    private boolean startsReported;

    private volatile double tps = Double.NaN;
    private volatile double mspt = Double.NaN;
    private volatile long lastTickAt;

    /**
     * Начало тика. Необязательно: без него считается всё, кроме {@link #mspt()}.
     *
     * <p>У модов есть и Pre, и Post; у Paper — только повторяющаяся задача, то
     * есть одна точка на тик. Врать средним по циклу вместо реального времени
     * работы мы не станем: цикл здорового сервера всегда 50 мс, и такая «mspt»
     * не сказала бы ничего сверх TPS.
     */
    public void onTickStart() {
        tickStart = System.nanoTime();
        startsReported = true;
    }

    /** Конец тика. Единственный обязательный вызов. */
    public void onTickEnd() {
        long now = System.nanoTime();
        lastTickAt = now;
        if (windowStart == 0) {
            windowStart = now;
            return;
        }
        windowTicks++;
        if (startsReported && tickStart != 0) {
            windowWorkNanos += now - tickStart;
        }
        long elapsed = now - windowStart;
        if (elapsed < WINDOW_NANOS) {
            return;
        }
        // Сервер не гонит быстрее 20 тиков в секунду; всё выше — округление.
        tps = Math.min(20.0, windowTicks * NANOS_PER_SECOND / elapsed);
        mspt = startsReported ? windowWorkNanos / NANOS_PER_MILLI / windowTicks : Double.NaN;
        windowStart = now;
        windowTicks = 0;
        windowWorkNanos = 0;
    }

    /** @return {@code null}, пока не набралось первое окно */
    public Double tps() {
        double value = tps;
        return Double.isNaN(value) ? null : round(value);
    }

    /** @return {@code null}, если платформа не сообщает начало тика */
    public Double mspt() {
        double value = mspt;
        return Double.isNaN(value) ? null : round(value);
    }

    /**
     * Сколько секунд игровой поток не двигался.
     *
     * <p>Ноль до первого тика: сервер ещё грузится, и считать это зависанием
     * значит перезапускать его посреди генерации мира.
     */
    public int stalledSeconds() {
        long last = lastTickAt;
        if (last == 0) {
            return 0;
        }
        return (int) ((System.nanoTime() - last) / NANOS_PER_SECOND);
    }

    private static double round(double value) {
        return Math.round(value * 10.0) / 10.0;
    }
}
