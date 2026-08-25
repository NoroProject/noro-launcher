package dev.noro.client.staff;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.toasts.Toast;
import net.minecraft.client.gui.components.toasts.ToastComponent;
import net.minecraft.network.chat.Component;

/**
 * Новое дело — тост, а не строка в чат.
 *
 * <p>Спам в чат модератор перестаёт читать на второй день. Уголок экрана и
 * клавиша «взять» — то же уведомление, но не мешающее играть. Звука нет
 * намеренно: панель ничем не должна выдавать разбор.
 */
public final class CaseToast implements Toast {

    /** Столько же, сколько ванильные тосты: дольше — это уже помеха. */
    private static final long SHOWN_MS = 5000L;

    /** Ванильная подложка: своя текстура ради одного тоста ничего не добавит. */
    private static final net.minecraft.resources.ResourceLocation BACKGROUND =
            net.minecraft.resources.ResourceLocation.withDefaultNamespace("toast/system");

    private final String target;
    private final String label;

    private CaseToast(String target, String label) {
        this.target = target;
        this.label = label;
    }

    /** Показать, если панель вообще у этого игрока есть. */
    public static void show(CaseModels.Brief brief) {
        if (!NoroStaff.state().can(NoroStaff.PERM_VIEW)) {
            return;
        }
        ToastComponent toasts = Minecraft.getInstance().getToasts();
        toasts.addToast(
                new CaseToast(brief.target_name() == null ? "—" : brief.target_name(), brief.label()));
    }

    @Override
    public Visibility render(GuiGraphics g, ToastComponent toasts, long shownFor) {
        g.blitSprite(BACKGROUND, 0, 0, width(), height());
        Font font = toasts.getMinecraft().font;
        g.drawString(font, Component.translatable("noro.cases.toast.new"), 8, 7, 0xFFE0C070, false);
        g.drawString(
                font,
                Component.translatable("noro.cases.toast.take", target),
                8,
                18,
                0xFFE6E6EA,
                false);
        g.drawString(font, label, width() - 8 - font.width(label), 7, 0xFF8A8A96, false);
        return shownFor < SHOWN_MS ? Visibility.SHOW : Visibility.HIDE;
    }
}
