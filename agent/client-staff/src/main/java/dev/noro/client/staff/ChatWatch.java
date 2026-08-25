package dev.noro.client.staff;

import dev.noro.client.staff.CaseIntents;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Instant;
import java.util.ArrayDeque;
import java.util.Deque;
import java.util.HexFormat;
import net.minecraft.network.chat.Component;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.neoforge.client.event.ClientChatReceivedEvent;

/**
 * Строки чата, которые модератор видел своими глазами.
 *
 * <p>Модератор, который видел мат, не должен ждать жалобы и искать окно во
 * времени. Но клиент не источник доказательств — он лишь указывает на них:
 * наружу уходят отправитель, время и хеш текста, а в дело едет строка из
 * буфера агента. Подделать переписку правым кликом поэтому невозможно.
 */
@EventBusSubscriber(modid = NoroStaff.ID, value = Dist.CLIENT)
public final class ChatWatch {

    /** Хватает на разбор по горячим следам; дальше отвечает буфер агента. */
    private static final int KEEP = 100;

    public record Line(String sender, String at, String text, String hash) {}

    private static final Deque<Line> LINES = new ArrayDeque<>();

    private ChatWatch() {}

    @SubscribeEvent
    public static void onChat(ClientChatReceivedEvent event) {
        String text = event.getMessage().getString();
        if (text.isEmpty()) {
            return;
        }
        synchronized (LINES) {
            LINES.addLast(new Line(sender(event.getMessage()), Instant.now().toString(), text,
                    hash(text)));
            while (LINES.size() > KEEP) {
                LINES.removeFirst();
            }
        }
    }

    /** Снимок буфера от старых к свежим — по нему считается строка под курсором. */
    public static java.util.List<Line> recent() {
        synchronized (LINES) {
            return java.util.List.copyOf(LINES);
        }
    }

    public static void quoteToOpenCase(Line line) {
        CaseModels.View view = NoroStaff.state().open();
        if (view == null || view.brief() == null || line == null) {
            return;
        }
        NoroStaff.cases()
                .send(new CaseIntents.Quote(
                        view.brief().id(), line.sender(), line.at(), line.hash()));
    }

    /**
     * Ник вытаскивается из начала строки: у ванильного формата это
     * {@code <ник> текст}, у серверного — префикс роли перед ним.
     */
    private static String sender(Component message) {
        String text = message.getString();
        int open = text.indexOf('<');
        int close = text.indexOf('>');
        if (open >= 0 && close > open) {
            return text.substring(open + 1, close).trim();
        }
        int colon = text.indexOf(':');
        return colon > 0 ? text.substring(0, colon).trim() : "";
    }

    /**
     * Отпечаток строки чата — по нему агент находит её в своём буфере.
     *
     * <p>Без запасной пустой строки: по ней сообщение не нашлось бы, и модератор
     * жал бы «процитировать в дело» без всякого результата и без объяснения.
     * SHA-1 обязателен для любой JVM, так что отказ здесь — это сломанная
     * платформа, а не случай, который надо обходить.
     */
    private static String hash(String text) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-1");
            return HexFormat.of().formatHex(digest.digest(text.getBytes(StandardCharsets.UTF_8)));
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException("в этой JVM нет SHA-1", e);
        }
    }
}
