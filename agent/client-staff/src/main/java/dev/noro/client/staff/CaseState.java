package dev.noro.client.staff;

import dev.noro.client.staff.CaseFrames;
import java.util.ArrayList;
import java.util.List;

/**
 * Что панель знает прямо сейчас.
 *
 * <p>Мод не «запрашивает и ждёт ответ» — он подписан: карточка обновляется сама,
 * когда с мастером что-то произошло. Поэтому состояние живёт здесь, а экраны
 * только читают его на отрисовке.
 */
public final class CaseState {

    private volatile CaseFrames.Ready ready;
    private volatile List<CaseModels.Brief> queue = List.of();
    /** Какой странице очереди соответствует то, что лежит в {@link #queue}. */
    private volatile long queueTotal;

    private volatile long queueOffset;
    private volatile CaseModels.View open;
    private volatile CaseModels.Dossier dossier;
    private volatile List<CaseModels.Slot> inventory = List.of();
    private volatile String inventoryCase;
    /** Последний отказ: экран гасит кнопку и показывает ключ перевода. */
    private volatile CaseFrames.Rejected rejected;

    private final List<Runnable> listeners = new ArrayList<>();

    /** Кадр от лаунчера. Забытый вид кадра не скомпилируется — на то и sealed. */
    public void accept(CaseFrames frame) {
        switch (frame) {
            case CaseFrames.Ready r -> ready = r;
            case CaseFrames.Queue q -> {
                List<CaseModels.Brief> fresh = q.cases() == null ? List.of() : q.cases();
                // Уведомляем только про начало неотфильтрованной очереди: на
                // второй странице и под поиском «новым» выглядит всё, чего не
                // было в предыдущей выдаче, то есть почти каждая строка.
                if (q.offset() == 0 && (q.query() == null || q.query().isBlank())) {
                    announce(fresh);
                }
                queue = fresh;
                queueTotal = q.total();
                queueOffset = q.offset();
            }
            case CaseFrames.Case c -> {
                open = c.view();
                rejected = null;
            }
            case CaseFrames.Dossier d -> dossier = d.dossier();
            case CaseFrames.Inventory i -> {
                inventoryCase = i.case_id();
                inventory = i.items() == null ? List.of() : i.items();
            }
            case CaseFrames.Rejected r -> rejected = r;
            case CaseFrames.Notice n -> NoroStaff.LOG.info("лаунчер: {}", n.key());
        }
        notifyListeners();
    }

    /**
     * Тост на каждое дело, которого в прошлой очереди не было.
     *
     * <p>Первая очередь после подключения молчит: показать десяток тостов при
     * входе на сервер — не уведомить, а завалить.
     */
    private void announce(List<CaseModels.Brief> fresh) {
        List<CaseModels.Brief> known = queue;
        if (known.isEmpty()) {
            return;
        }
        for (CaseModels.Brief c : fresh) {
            boolean seen = known.stream().anyMatch(k -> k.id().equals(c.id()));
            if (!seen && c.claimed_by() == null) {
                CaseToast.show(c);
            }
        }
    }

    public void disconnected() {
        ready = null;
        open = null;
        queue = List.of();
        queueTotal = 0;
        queueOffset = 0;
        notifyListeners();
    }

    /** Экран перерисовывается по изменению, а не по таймеру. */
    public void onChange(Runnable listener) {
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    private void notifyListeners() {
        synchronized (listeners) {
            for (Runnable l : listeners) {
                l.run();
            }
        }
    }

    public CaseFrames.Ready ready() {
        return ready;
    }

    /** Есть ли право. Кнопки рисуются по нему; решает всё равно мастер. */
    public boolean can(String permission) {
        CaseFrames.Ready r = ready;
        return r != null && r.has(permission);
    }

    public List<CaseModels.Brief> queue() {
        return queue;
    }

    /** Сколько дел подходит под текущий фильтр целиком, а не сколько на экране. */
    public long queueTotal() {
        return queueTotal;
    }

    /** Сдвиг показанной страницы от начала очереди. */
    public long queueOffset() {
        return queueOffset;
    }

    public CaseModels.View open() {
        return open;
    }

    public CaseModels.Dossier dossier() {
        return dossier;
    }

    public List<CaseModels.Slot> inventory(String caseId) {
        return caseId != null && caseId.equals(inventoryCase) ? inventory : List.of();
    }

    public CaseFrames.Rejected rejected() {
        return rejected;
    }

    public void clearRejected() {
        rejected = null;
    }
}
