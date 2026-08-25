package dev.noro.client.ui;

import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Палитра лаунчера в игре.
 *
 * <p>Значения — те же, что в {@code crates/frontend/src/theme.rs}: экран мода и
 * экран лаунчера должны выглядеть одним продуктом, а не двумя похожими. Цвета
 * берутся только отсюда — ни одного литерала в экранах.
 *
 * <p>Шаг сетки 4 пикселя, как во всём остальном проекте: любой размер и отступ
 * кратен {@link #GRID}.
 */
public final class Theme {

    private Theme() {}

    public static final int GRID = 4;

    /** Полупрозрачный слой поверх игры: панель не должна слепить в ванише. */
    public static final int OVERLAY = 0xD0081020;

    public static final int BG_PANEL = 0xFF13233D;
    public static final int BG_CARD = 0xFF172A47;
    public static final int BG_CARD_HOVER = 0xFF1F3556;
    public static final int BG_INPUT = 0xFF0F2036;
    public static final int BORDER = 0xFF223A55;

    /** Рамка под курсором: та же линия, но заметнее. */
    public static final int BORDER_STRONG = 0xFF2F4E70;

    /** Главное действие — тёплый кремовый, как кнопка «Играть» в лаунчере. */
    public static final int CTA = 0xFFF3E7B3;
    public static final int CTA_HOVER = 0xFFFBF0C4;
    public static final int ON_CTA = 0xFF12233D;
    public static final int ACCENT = 0xFFE85AA5;
    public static final int BLUE = 0xFF7FB2FF;

    public static final int SUCCESS = 0xFF7EE0A4;
    public static final int WARNING = 0xFFF3C969;
    public static final int ERROR = 0xFFFF6B8B;

    public static final int TEXT = 0xFFDBE6FF;
    public static final int TEXT_SECONDARY = 0xFF9FB0D6;
    public static final int TEXT_MUTED = 0xFF5A6B91;

    /** Тень под плашкой HUD: без неё панель лежит в мире, а не над ним. */
    public static final int SHADOW = 0x50000000;

    /** Внутренний блик рамки: он и делает край объёмным, а не нарисованным. */
    public static final int EDGE = 0xFF1D3455;

    /**
     * Тот же цвет, но светлее на {@code step} по каждому каналу.
     *
     * <p>Нужен для блика по верхней кромке: держать вторую константу на каждый
     * оттенок кнопки — четыре палитры вместо одной, и они разъедутся.
     */
    public static int lift(int color, int step) {
        int r = Math.min(255, (color >> 16 & 0xFF) + step);
        int g = Math.min(255, (color >> 8 & 0xFF) + step);
        int b = Math.min(255, (color & 0xFF) + step);
        return (color & 0xFF000000) | r << 16 | g << 8 | b;
    }

    // Кольцо радиального меню: полупрозрачное, чтобы не закрывать мир под ним.
    public static final int RING = 0xB013233D;
    public static final int RING_ACTIVE = 0xF0F3E7B3;
    public static final int RING_DISABLED = 0x800F2036;

    /**
     * Метка над полем: заглавными и приглушённым, как `noro-label` на сайте.
     *
     * <p>Разрядка пробелом — единственный способ получить в игровом шрифте тот
     * же воздух, что даёт `letter-spacing` в вебе. Без неё блок подписей
     * сливается в сплошную строку и перестаёт делить форму на части.
     */
    public static int label(GuiGraphics g, Component text, int x, int y) {
        g.drawString(font(), text.getString().toUpperCase(java.util.Locale.ROOT),
                x, y, TEXT_MUTED, false);
        return y + 3 * GRID;
    }

    /**
     * Обрезать строку по ширине, дописав многоточие.
     *
     * <p>Без него обрезка выглядела поломкой: «Уважение к другим игрок» читается
     * как опечатка, а «Уважение к другим…» — как «здесь есть ещё».
     */
    public static String clip(Component text, int w) {
        var font = font();
        String value = text.getString();
        if (font.width(value) <= w) {
            return value;
        }
        return font.plainSubstrByWidth(value, w - font.width("…")) + "…";
    }

    /** Шрифт клиента: он один на всё, и тянуть его через Minecraft каждый раз — шум. */
    public static net.minecraft.client.gui.Font font() {
        return net.minecraft.client.Minecraft.getInstance().font;
    }

}
