package dev.noro.client.ui;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import com.mojang.blaze3d.vertex.DefaultVertexFormat;
import com.mojang.blaze3d.vertex.Tesselator;
import com.mojang.blaze3d.vertex.VertexFormat;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.renderer.GameRenderer;
import org.joml.Matrix4f;

/**
 * Кольцевой сектор — из них собирается радиальное меню.
 *
 * <p>{@code GuiGraphics.fill} умеет только прямоугольники по осям, поэтому
 * сектор кольца рисуется треугольниками напрямую. Приближать дугу лесенкой из
 * прямоугольников можно, но на глаз это видно сразу, а меню — то, на что
 * модератор смотрит десятки раз за смену.
 */
public final class Ring {

    /** Шаг разбиения дуги. Два градуса — край уже читается как гладкий. */
    private static final double STEP = Math.toRadians(2);

    private Ring() {}

    /**
     * Залить сектор от {@code from} до {@code to} (радианы, ноль сверху, по
     * часовой) между внутренним и внешним радиусом.
     */
    public static void sector(
            GuiGraphics g,
            float cx,
            float cy,
            float inner,
            float outer,
            double from,
            double to,
            int argb) {
        RenderSystem.enableBlend();
        RenderSystem.defaultBlendFunc();
        RenderSystem.setShader(GameRenderer::getPositionColorShader);

        Matrix4f matrix = g.pose().last().pose();
        BufferBuilder buffer = Tesselator.getInstance()
                .begin(VertexFormat.Mode.TRIANGLE_STRIP, DefaultVertexFormat.POSITION_COLOR);

        int steps = Math.max(2, (int) Math.ceil((to - from) / STEP));
        for (int i = 0; i <= steps; i++) {
            double angle = from + (to - from) * i / steps;
            // Ноль сверху и по часовой: так сектор совпадает с направлением,
            // куда игрок повёл мышью, без пересчёта в голове.
            float sin = (float) Math.sin(angle);
            float cos = (float) Math.cos(angle);
            buffer.addVertex(matrix, cx + sin * inner, cy - cos * inner, 0).setColor(argb);
            buffer.addVertex(matrix, cx + sin * outer, cy - cos * outer, 0).setColor(argb);
        }
        BufferUploader.drawWithShader(buffer.buildOrThrow());
        RenderSystem.disableBlend();
    }
}
