package dev.noro.client.player;

import net.neoforged.api.distmarker.Dist;
import net.neoforged.fml.common.Mod;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * То, что нужно обычному игроку: свод правил, свои наказания, жалоба формой.
 *
 * <p>Отдельный мод от staff, потому что раздаётся иначе: этот едет всем, а
 * панель разбора — по праву. Игрок не должен получать половину инструментов
 * модератора вместе с обычной сборкой, а модератор — тащить игрокские экраны.
 *
 * <p>Своего канала у мода нет: соединение с лаунчером держит ядро, а функции
 * подключаются к нему через {@code NoroCore.register}.
 */
@Mod(value = NoroPlayer.ID, dist = Dist.CLIENT)
public final class NoroPlayer {

    public static final String ID = "noro_player";
    public static final Logger LOG = LoggerFactory.getLogger("NoroPlayer");

    public NoroPlayer() {
        // Перезагрузка наборов: лаунчер подменяет паки под работающей игрой, а
        // позвать перезагрузку можно только изнутри неё. Касается всех игроков,
        // поэтому здесь, а не в моде модератора.
        dev.noro.client.NoroCore.register(new Resources());
    }
}
