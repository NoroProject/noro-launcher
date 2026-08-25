package dev.noro.client.staff;

import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.components.ChatComponent;
import net.minecraft.network.chat.Component;

/**
 * Какая строка чата под курсором.
 *
 * <p>Геометрию чата Minecraft держит при себе, но она вся выводится из
 * публичных настроек: ширина, масштаб и межстрочный интервал. Считаем перенос
 * своим шрифтом по своему же буферу — так индекс совпадает со строкой, которую
 * увидел модератор, даже когда длинное сообщение заняло три ряда.
 */
public final class ChatPick {

    /** Отступ чата от низа экрана — тот же, что в {@code screenToChatY}. */
    private static final double BOTTOM = 40.0;

    /** Базовая высота строки шрифта; интервал добавляет настройка игрока. */
    private static final double LINE = 9.0;

    private ChatPick() {}

    /** {@code null} — под курсором пусто или чат пуст. */
    public static ChatWatch.Line at(double mouseX, double mouseY) {
        Minecraft mc = Minecraft.getInstance();
        ChatComponent chat = mc.gui.getChat();
        double scale = chat.getScale();
        if (scale <= 0) {
            return null;
        }
        double lineHeight = LINE * (mc.options.chatLineSpacing().get() + 1.0);
        double fromBottom = mc.getWindow().getGuiScaledHeight() - mouseY - BOTTOM;
        int row = (int) (fromBottom / (scale * lineHeight));
        if (row < 0 || row >= chat.getHeight() / lineHeight + 1) {
            return null;
        }

        // Ряды считаются снизу вверх, сообщения — от свежих к старым.
        List<ChatWatch.Line> lines = ChatWatch.recent();
        int width = (int) (chat.getWidth() / scale);
        int consumed = 0;
        for (int i = lines.size() - 1; i >= 0; i--) {
            ChatWatch.Line line = lines.get(i);
            int wrapped = Math.max(1, mc.font.split(Component.literal(line.text()), width).size());
            if (row < consumed + wrapped) {
                return line;
            }
            consumed += wrapped;
        }
        return null;
    }
}
