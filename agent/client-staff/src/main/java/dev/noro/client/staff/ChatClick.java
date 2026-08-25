package dev.noro.client.staff;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.ChatScreen;
import net.minecraft.network.chat.Component;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.neoforge.client.event.ScreenEvent;

/**
 * Правый клик по строке чата — «в дело».
 *
 * <p>Модератор, который видел мат своими глазами, не должен ждать жалобы и
 * искать окно во времени. Наружу при этом уходят только отправитель, время и
 * хеш: строку в дело кладёт агент из своего буфера.
 */
@EventBusSubscriber(modid = NoroStaff.ID, value = Dist.CLIENT)
public final class ChatClick {

    /** GLFW: правая кнопка мыши. */
    private static final int RIGHT = 1;

    private ChatClick() {}

    @SubscribeEvent
    public static void onClick(ScreenEvent.MouseButtonPressed.Pre event) {
        if (event.getButton() != RIGHT || !(event.getScreen() instanceof ChatScreen)) {
            return;
        }
        if (!NoroStaff.state().can(NoroStaff.PERM_CHAT)) {
            return;
        }
        ChatWatch.Line line = ChatPick.at(event.getMouseX(), event.getMouseY());
        if (line == null) {
            return;
        }
        ChatWatch.quoteToOpenCase(line);
        say(line);
        event.setCanceled(true);
    }

    /**
     * Подтверждение уходит в свой же чат: без него непонятно, попала строка в
     * дело или клик прошёл мимо. Дела нет — так и говорим, а не молчим.
     */
    private static void say(ChatWatch.Line line) {
        Minecraft mc = Minecraft.getInstance();
        if (mc.player == null) {
            return;
        }
        boolean hasCase = NoroStaff.state().open() != null;
        mc.player.displayClientMessage(
                Component.translatable(
                        hasCase ? "noro.cases.quote.sent" : "noro.cases.quote.nocase",
                        line.sender()),
                true);
    }
}
