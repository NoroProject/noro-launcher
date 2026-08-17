package dev.noro.agent.core;

import java.util.Locale;
import java.util.Set;

/**
 * Команды, которыми замученный обошёл бы мут.
 *
 * <p>Мут, который затыкает только общий чат, не мут: с {@code /msg} и
 * {@code /me} наказанный продолжает разговаривать. Полного списка «команд,
 * которые говорят» не существует — его знает только владелец сборки, поэтому
 * здесь ванильный и общеизвестный минимум.
 *
 * <p>Список общий для трёх платформ намеренно: разъехавшись, он дал бы мут,
 * который на Fabric строже, чем на Paper.
 */
public final class ChatCommands {

    private static final Set<String> BLOCKED = Set.of(
            "me", "msg", "tell", "w", "whisper", "say", "r", "reply", "m", "t", "pm", "emote", "broadcast");

    private ChatCommands() {}

    /**
     * @param line строка команды как её набрал игрок, со слэшем или без
     */
    public static boolean speaks(String line) {
        if (line == null) {
            return false;
        }
        String command = line.strip();
        if (command.startsWith("/")) {
            command = command.substring(1);
        }
        int space = command.indexOf(' ');
        if (space > 0) {
            command = command.substring(0, space);
        }
        // Плагины регистрируют команды и с префиксом своего имени:
        // `/essentials:msg` обошёл бы проверку по короткому имени.
        int colon = command.lastIndexOf(':');
        if (colon >= 0) {
            command = command.substring(colon + 1);
        }
        return BLOCKED.contains(command.toLowerCase(Locale.ROOT));
    }
}
