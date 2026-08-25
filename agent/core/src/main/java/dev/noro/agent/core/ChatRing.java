package dev.noro.agent.core;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.UUID;

/**
 * Последние сообщения сервера — в памяти и только в памяти.
 *
 * <p>Общего чат-лога у проекта нет сознательно: писать в базу каждую реплику
 * ради одной жалобы в неделю значит завести отдельную заботу о хранении и
 * приватности. Вместо этого агент держит короткое окно и отдаёт срез, когда
 * появился повод — жалоба или наказание.
 *
 * <p>Команды аутентификации сюда не попадают вовсе. Отфильтровать их позже, на
 * мастере или в админке, уже поздно: пароль к этому моменту побывал в канале.
 */
public final class ChatRing {

    /** Примерно десять минут разговора на живом сервере. */
    private static final int CAPACITY = 300;

    /** После них идёт пароль, а не сообщение. */
    private static final Set<String> SECRET = Set.of(
            "login", "l", "log", "register", "reg", "changepassword", "changepass", "cp");

    public record Entry(Instant at, UUID sender, String senderName, String channel, String content) {}

    private final ArrayDeque<Entry> entries = new ArrayDeque<>();

    /** Сообщение в чат: общий, локальный или личный. */
    public synchronized void message(UUID sender, String senderName, String channel, String content) {
        push(new Entry(Instant.now(), sender, senderName, channel, content));
    }

    /**
     * Выполненная команда. Пароль остаётся у игрока: команду из списка не
     * пишем ни целиком, ни первым словом — по «/login» тоже видно, кто где.
     */
    public synchronized void command(UUID sender, String senderName, String line) {
        if (isSecret(line)) {
            return;
        }
        push(new Entry(Instant.now(), sender, senderName, "command", line));
    }

    /** Срез за последние {@code window} секунд. */
    public synchronized List<Entry> slice(Duration window) {
        Instant from = Instant.now().minus(window);
        List<Entry> out = new ArrayList<>();
        for (Entry entry : entries) {
            if (!entry.at().isBefore(from)) {
                out.add(entry);
            }
        }
        return out;
    }

    private void push(Entry entry) {
        if (entries.size() >= CAPACITY) {
            entries.removeFirst();
        }
        entries.addLast(entry);
    }

    private static boolean isSecret(String line) {
        String text = line.startsWith("/") ? line.substring(1) : line;
        int space = text.indexOf(' ');
        String head = (space < 0 ? text : text.substring(0, space)).toLowerCase(Locale.ROOT);
        int colon = head.lastIndexOf(':'); // `minecraft:login`
        return SECRET.contains(colon < 0 ? head : head.substring(colon + 1));
    }
}
