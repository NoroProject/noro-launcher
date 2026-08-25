package dev.noro.client.staff.ui;

import com.google.gson.JsonParser;
import com.mojang.serialization.JsonOps;
import dev.noro.client.staff.CaseModels;
import dev.noro.client.ui.Surface;
import dev.noro.client.ui.Theme;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;
import net.minecraft.world.item.ItemStack;

/**
 * Снимок инвентаря цели — сеткой настоящих слотов.
 *
 * <p>Предмет приезжает целиком в JSON, поэтому рисуется он же: с зачарованием,
 * прочностью и переименованием. Список строк «предмет xN» этого не показывал,
 * а именно по нему в грифе и видно, чьё это было.
 */
final class Inventory {

    /** Иконка предмета — всегда 16 пикселей, это ванильная константа. */
    private static final int ICON = 16;

    /** Рамка слота: иконка плюс по пикселю с каждой стороны. */
    private static final int BOX = ICON + 2;

    /** Шаг сетки: рамка плюс зазор, чтобы слоты не слипались. */
    private static final int CELL = BOX + Theme.GRID / 2;

    private Inventory() {}

    static void render(
            GuiGraphics g, Font font, List<CaseModels.Slot> items, int x, int y, int w, int bottom) {
        if (items.isEmpty()) {
            g.drawString(font, Component.translatable("noro.cases.card.inventory.empty"),
                    x, y, Theme.TEXT_MUTED);
            return;
        }
        int columns = Math.max(1, w / CELL);
        int rows = Math.max(1, (bottom - y) / CELL);
        for (int i = 0; i < items.size() && i < columns * rows; i++) {
            int cx = x + (i % columns) * CELL;
            int cy = y + (i / columns) * CELL;
            cell(g, font, items.get(i), cx, cy);
        }
    }

    private static void cell(GuiGraphics g, Font font, CaseModels.Slot slot, int x, int y) {
        // Подложка слота: без неё предметы висят в пустоте и не читаются сеткой.
        g.fill(x, y, x + BOX, y + BOX, Theme.BG_INPUT);
        Surface.outline(g, x, y, BOX, BOX, Theme.BORDER);

        ItemStack stack = decode(slot);
        if (stack.isEmpty()) {
            // Предмет не восстановился — показываем хотя бы имя, а не пустоту.
            g.drawString(font, font.plainSubstrByWidth(slot.name(), BOX),
                    x + 2, y + 2, Theme.TEXT_MUTED, false);
            return;
        }
        // Ровно внутрь рамки: иконка 16, рамка 18, значит отступ ровно в пиксель.
        g.renderItem(stack, x + 1, y + 1);
        g.renderItemDecorations(font, stack, x + 1, y + 1);
    }

    /**
     * Предмет из JSON. Кодек тянет реестры — контекст берём у мира, иначе
     * зачарования и эффекты не разложить.
     */
    private static ItemStack decode(CaseModels.Slot slot) {
        Minecraft mc = Minecraft.getInstance();
        if (slot.nbt() == null || mc.level == null) {
            return ItemStack.EMPTY;
        }
        try {
            var ops = mc.level.registryAccess().createSerializationContext(JsonOps.INSTANCE);
            return ItemStack.CODEC
                    .parse(ops, JsonParser.parseString(slot.nbt()))
                    .result()
                    .orElse(ItemStack.EMPTY);
        } catch (Exception e) {
            return ItemStack.EMPTY;
        }
    }

    /** Подсказка под курсором: имя и количество, когда иконки мало. */
    static Component tooltip(CaseModels.Slot slot) {
        return Component.literal(slot.name() + " x" + slot.count());
    }
}
