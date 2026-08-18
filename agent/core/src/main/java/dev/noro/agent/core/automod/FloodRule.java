package dev.noro.agent.core.automod;

import java.util.ArrayDeque;
import java.util.Deque;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Фильтр флуда: отслеживание частоты отправки сообщений от конкретного игрока.
 */
public final class FloodRule {

    private static class MessageEntry {
        final long time;
        final String text;

        MessageEntry(long time, String text) {
            this.time = time;
            this.text = text;
        }
    }

    private final Map<UUID, Deque<MessageEntry>> playerHistory = new ConcurrentHashMap<>();

    public boolean check(UUID playerUuid, String text, FilterConfig config) {
        if (config == null || !config.enabled()) {
            return false;
        }

        int maxMessages = config.maxMessages() > 0 ? config.maxMessages() : 3;
        int windowMs = (config.windowSecs() > 0 ? config.windowSecs() : 4) * 1000;
        long now = System.currentTimeMillis();

        Deque<MessageEntry> history = playerHistory.computeIfAbsent(playerUuid, k -> new ArrayDeque<>());

        synchronized (history) {
            while (!history.isEmpty() && now - history.peekFirst().time > windowMs) {
                history.pollFirst();
            }

            int sameCount = 0;
            for (MessageEntry entry : history) {
                if (entry.text.equalsIgnoreCase(text)) {
                    sameCount++;
                }
            }

            history.addLast(new MessageEntry(now, text));

            return history.size() >= maxMessages || sameCount >= 2;
        }
    }
}
