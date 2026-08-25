package dev.noro.agent.core;

import java.time.Duration;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Режим разбора жалобы в игре.
 *
 * <p>Состояние держим здесь, а не на мастере: меню и телепорты нужны мгновенно,
 * а рвущийся канал не должен оставлять модератора без кнопок. Мастер при этом
 * остаётся хозяином замка — кто ведёт дело, решает он, и агент лишь исполняет
 * присланное.
 */
public final class CaseMode {

    /** Сколько чата отматывать назад по просьбе мастера. */
    private static final Duration DEFAULT_WINDOW = Duration.ofMinutes(10);

    private final GameBridge game;
    private final AgentEvents events;
    private final ChatRing chat;
    private final VanishBridge vanish;

    /**
     * Разборы модератора: он может вести несколько дел разом.
     *
     * <p>Раньше здесь было одно дело на человека, и это расходилось с мастером,
     * где замков у одного модератора бывает сколько угодно. Панель показывала
     * одно дело, а команда уходила в другое — то, которое агент запомнил
     * последним.
     */
    private final Map<UUID, Map<UUID, CaseSession>> sessions = new ConcurrentHashMap<>();

    /** Какое из них сейчас на экране: его и имеют в виду команды без номера. */
    private final Map<UUID, UUID> active = new ConcurrentHashMap<>();

    /** Ваниш модератора на время разбора: платформа умеет это по-своему. */
    @FunctionalInterface
    public interface VanishBridge {
        void vanish(UUID who, boolean on);
    }

    public CaseMode(GameBridge game, AgentEvents events, ChatRing chat, VanishBridge vanish) {
        this.game = game;
        this.events = events;
        this.chat = chat;
        this.vanish = vanish;
    }

    public Optional<CaseSession> session(UUID moderator) {
        UUID current = active.get(moderator);
        Map<UUID, CaseSession> mine = sessions.get(moderator);
        if (current == null || mine == null) {
            return Optional.empty();
        }
        return Optional.ofNullable(mine.get(current));
    }

    /**
     * Переключиться на другое своё дело.
     *
     * <p>Панель зовёт это, открывая карточку: тогда «телепорт» и «заморозка»
     * попадают в то дело, которое модератор видит, а не в то, которое агент
     * запомнил последним.
     *
     * @return {@code false} — такого дела за модератором нет
     */
    public boolean use(UUID moderator, UUID caseId) {
        Map<UUID, CaseSession> mine = sessions.get(moderator);
        if (mine == null || !mine.containsKey(caseId)) {
            return false;
        }
        active.put(moderator, caseId);
        return true;
    }

    public ChatRing chat() {
        return chat;
    }

    /**
     * Мастер отдал дело модератору: прячем его, подсвечиваем участников и
     * показываем меню. Модератора может не быть в игре — тогда кадр просто
     * ничего не делает, и это нормальный случай.
     */
    public void assigned(CaseSession incoming, UUID moderator, String lang) {
        CaseSession session = incoming.watching(false);
        GameBridge.Position origin = game.position(moderator).orElse(null);
        session = new CaseSession(
                session.caseId(), session.target(), session.targetName(), session.reporter(),
                session.reporterName(), session.reason(), session.world(), session.x(), session.y(),
                session.z(), origin, false);
        sessions.computeIfAbsent(moderator, k -> new ConcurrentHashMap<>())
                .put(session.caseId(), session);
        active.put(moderator, session.caseId());

        vanish.vanish(moderator, true);
        game.glow(moderator, session.target(), true);
        if (session.reporter() != null) {
            game.glow(moderator, session.reporter(), true);
        }
        game.tell(moderator, CaseMenu.render(session, lang));
    }

    /** Дело отпустили или закрыли — вернуть модератора как было. */
    public void finished(UUID moderator, UUID caseId, boolean closed, String lang) {
        Map<UUID, CaseSession> mine = sessions.get(moderator);
        CaseSession session = mine == null ? null : mine.remove(caseId);
        if (session == null) {
            return;
        }
        // Закрыли то, что было на экране — переводим на любое оставшееся, а не
        // оставляем модератора «ни в каком деле», пока у него есть другие.
        if (caseId.equals(active.get(moderator))) {
            active.remove(moderator);
            if (mine != null && !mine.isEmpty()) {
                active.put(moderator, mine.keySet().iterator().next());
            }
        }
        game.glow(moderator, session.target(), false);
        if (session.reporter() != null) {
            game.glow(moderator, session.reporter(), false);
        }
        if (session.watching()) {
            game.stopSpectate(moderator);
        }
        vanish.vanish(moderator, false);
        if (session.origin() != null) {
            GameBridge.Position at = session.origin();
            game.teleport(moderator, at.world(), at.x(), at.y(), at.z());
        }
        game.tell(moderator, AgentStrings.get(lang, closed ? "case_closed" : "case_released"));
    }

    /** Заменить состояние разбора: слежку включают и выключают на ходу. */
    void replace(UUID moderator, CaseSession session) {
        Map<UUID, CaseSession> mine = sessions.get(moderator);
        if (mine != null) {
            mine.replace(session.caseId(), session);
        }
    }

    /** Срез чата по просьбе мастера — окно, а не весь буфер. */
    public void sendChatSlice(UUID caseId, int beforeSecs) {
        Duration window = beforeSecs > 0 ? Duration.ofSeconds(beforeSecs) : DEFAULT_WINDOW;
        events.caseChatSlice(caseId, chat.slice(window));
    }

    public void sendInventory(UUID caseId, UUID target, UUID moderator) {
        List<GameBridge.Slot> items = game.inventory(target);
        events.caseInventory(caseId, moderator, items);
    }
}
