package dev.noro.agent.paper;

import dev.noro.agent.core.PlaceholderValues;
import dev.noro.agent.core.PlayerProfile;
import dev.noro.agent.core.ProfileCache;
import me.clip.placeholderapi.expansion.PlaceholderExpansion;
import org.bukkit.OfflinePlayer;
import org.bukkit.plugin.Plugin;

/**
 * Профиль мастера в виде {@code %noro_<ключ>%} для PlaceholderAPI.
 *
 * <p>Класс грузится только после того, как {@link PlaceholderSupport} убедился
 * в наличии PlaceholderAPI: без него {@link PlaceholderExpansion} не найдётся
 * и загрузка упадёт с NoClassDefFoundError.
 */
final class PaperPlaceholders extends PlaceholderExpansion {

    private final Plugin plugin;
    private final ProfileCache profiles;

    PaperPlaceholders(Plugin plugin, ProfileCache profiles) {
        this.plugin = plugin;
        this.profiles = profiles;
    }

    @Override
    public String getIdentifier() {
        return "noro";
    }

    @Override
    public String getAuthor() {
        return "noro";
    }

    /**
     * {@code getDescription()} на свежих Paper помечен устаревшим в пользу
     * {@code getPluginMeta()}, но того нет до 1.19.3, а исходник у всех сборок
     * общий и препроцессора здесь нет. Устаревший метод работает на всём
     * диапазоне — это и есть меньшее из двух зол.
     */
    @Override
    @SuppressWarnings("deprecation")
    public String getVersion() {
        return plugin.getDescription().getVersion();
    }

    /**
     * Расширение живёт вместе с агентом, а не с PlaceholderAPI: без этого
     * {@code /papi reload} снимает его и все плейсхолдеры разом становятся
     * текстом, пока сервер не перезапустят.
     */
    @Override
    public boolean persist() {
        return true;
    }

    /**
     * @return {@code null} на чужой ключ — тогда PlaceholderAPI оставит текст
     *         как есть, и ключ достанется тому расширению, чей он на самом деле
     */
    @Override
    public String onRequest(OfflinePlayer player, String params) {
        if (player == null) {
            return null;
        }
        // Профиль есть только у тех, кто сейчас на сервере: агент забирает его
        // на входе и отпускает на выходе. Для офлайнового игрока честнее
        // промолчать, чем показать данные, устаревшие на неизвестный срок.
        PlayerProfile profile = profiles.get(player.getUniqueId());
        return PlaceholderValues.resolve(profile, params);
    }
}
