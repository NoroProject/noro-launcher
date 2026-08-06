package dev.noro.agent.core;

import java.util.Collection;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentSkipListSet;
import java.util.function.Consumer;

/**
 * Каталог узлов прав, который агент отдаёт мастеру для подсказок в админке.
 *
 * <p>Существует потому, что реестр узлов есть не везде. На Paper их перечисляет
 * {@code PluginManager}, на Forge и NeoForge — {@code PermissionGatherEvent}, а
 * на Fabric такого реестра нет ни у одного мода: спросить там некого, и без
 * этого класса каталог с Fabric-сервера всегда оставался бы пустым. Сюда мод
 * кладёт свои узлы сам, одним вызовом и на любой платформе.
 *
 * <p>Порядок вызовов значения не имеет. Узлы, пришедшие до того, как платформа
 * подключила отправку, ждут её в наборе; пришедшие после — уходят сразу. Иначе
 * пришлось бы навязывать чужим модам момент регистрации, а он у каждого лоадера
 * свой и меняется от версии к версии.
 *
 * <p>Набор конкурентный и отсортированный: пишут в него разные моды из разных
 * потоков старта, а мастер получает стабильный порядок — иначе в админке список
 * прыгал бы между перезапусками.
 */
public final class PermissionNodeCatalog {

    private final Set<String> nodes = new ConcurrentSkipListSet<>();

    /** Транспорт до мастера. Появляется, когда платформа готова отправлять. */
    private volatile Consumer<Collection<String>> sink;

    /** Есть ли что отправлять: набор мог не измениться со времени прошлой отправки. */
    private volatile boolean dirty;

    PermissionNodeCatalog() {}

    /**
     * Добавляет узлы в каталог.
     *
     * <p>Повторные вызовы с теми же узлами ничего не стоят: набор их поглотит и
     * отправки не будет. Это важно для перезагружаемых плагинов — они шлют свой
     * список на каждый {@code reload}.
     */
    public void register(Collection<String> newNodes) {
        if (newNodes == null || newNodes.isEmpty()) {
            return;
        }
        if (nodes.addAll(newNodes)) {
            dirty = true;
            flush();
        }
    }

    /** Всё, что накопилось. Копия: набор продолжает жить и меняться. */
    public List<String> nodes() {
        return List.copyOf(nodes);
    }

    /**
     * Подключает отправку и сразу выкладывает всё, что успело накопиться.
     *
     * <p>Зовёт платформенная часть агента, когда узнала собственный реестр:
     * до этого момента отправлять неполный каталог смысла нет — мастер хранит
     * последний присланный список целиком.
     */
    public void attach(Consumer<Collection<String>> transport) {
        this.sink = transport;
        dirty = true;
        flush();
    }

    /**
     * Отправляет каталог, если есть что и есть куда.
     *
     * <p>Отправка синхронная по отношению к вызывающему: транспорт сам решает,
     * уводить ли её с главного потока. Здесь про потоки ничего не известно, а
     * гадать — значит однажды повесить старт сервера на сеть.
     */
    private void flush() {
        Consumer<Collection<String>> transport = sink;
        if (transport == null || !dirty) {
            return;
        }
        dirty = false;
        transport.accept(nodes());
    }

}
