package dev.noro.client;

import dev.noro.client.link.Bridge;
import dev.noro.client.link.Feature;
import dev.noro.client.rules.RuleBook;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import net.minecraft.client.Minecraft;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.fml.common.Mod;
import net.neoforged.neoforge.client.event.ClientPlayerNetworkEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Ядро клиентских модов Noro: канал с лаунчером и общий вид.
 *
 * <p>Само по себе ядро ничего не показывает. Оно держит одно соединение с
 * лаунчером, раздаёт кадры функциям и даёт им экраны, кнопки и плашки в стиле
 * лаунчера, чтобы «в стиле Noro» не переписывалось в каждом моде заново.
 *
 * <p>Поверх ядра живут отдельные моды: {@code noro_staff} — инструменты
 * модератора, {@code noro_player} — то, что нужно игроку. Разделены они не для
 * красоты: staff раздаётся по праву и лежит не у всех, а игрок не должен
 * получать половину панели разбора вместе с обычной сборкой.
 *
 * <p>Ни один из них не носитель прав. Клиент подделывается, jar
 * декомпилируется; любая проверка внутри него украшение. Мастер отвечает 403 —
 * экран показывает отказ, и это единственная линия.
 */
@Mod(value = NoroCore.ID, dist = Dist.CLIENT)
@EventBusSubscriber(modid = NoroCore.ID, value = Dist.CLIENT)
public final class NoroCore {

    public static final String ID = "noro_core";
    public static final Logger LOG = LoggerFactory.getLogger("NoroCore");

    /**
     * Функции регистрируются на загрузке своих модов, до входа на сервер.
     * Список копируемый: регистрация идёт с потока загрузки, а раздача кадров —
     * с потока сокета.
     */
    private static final List<Feature> FEATURES = new CopyOnWriteArrayList<>();

    /**
     * Свод правил подключается самим ядром: он нужен и модератору при выдаче
     * наказания, и игроку — прочитать, за что наказали. Держать его копию в
     * каждом моде значит однажды показать два разных свода.
     */
    private static final RuleBook RULES = new RuleBook();

    private static final Bridge BRIDGE = new Bridge(FEATURES);

    public NoroCore() {}

    static {
        FEATURES.add(RULES);
    }

    public static RuleBook rules() {
        return RULES;
    }

    /** Подключить свою функцию к каналу. Вызывается из конструктора мода. */
    public static void register(Feature feature) {
        FEATURES.add(feature);
        LOG.info("функция подключена: {}", feature.id());
    }

    public static Bridge bridge() {
        return BRIDGE;
    }

    /** Вход на сервер — момент, когда файл рукопожатия уже на месте. */
    @SubscribeEvent
    public static void onJoin(ClientPlayerNetworkEvent.LoggingIn event) {
        BRIDGE.connect(Minecraft.getInstance().gameDirectory.toPath());
    }

    @SubscribeEvent
    public static void onLeave(ClientPlayerNetworkEvent.LoggingOut event) {
        BRIDGE.close();
    }
}
