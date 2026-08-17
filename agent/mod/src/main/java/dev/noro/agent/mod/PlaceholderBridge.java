package dev.noro.agent.mod;

import dev.noro.agent.core.PlaceholderValues;
import dev.noro.agent.core.ProfileCache;
//#if FABRIC==1
import java.util.UUID;
//#if MC>=260000
//$$ import eu.pb4.placeholders.api.PlaceholderResult;
//$$ import eu.pb4.placeholders.api.Placeholders;
//$$ import net.minecraft.resources.Identifier;
//#elseif MC>=11900
import eu.pb4.placeholders.api.PlaceholderResult;
import eu.pb4.placeholders.api.Placeholders;
import net.minecraft.resources.ResourceLocation;
//#else
//$$ import eu.pb4.placeholders.PlaceholderAPI;
//$$ import eu.pb4.placeholders.PlaceholderResult;
//$$ import net.minecraft.resources.ResourceLocation;
//#endif
//#endif

/**
 * Регистрация {@code %noro:<ключ>%} в Text Placeholder API.
 *
 * <p>Текст этого класса виден компилятору только на Fabric: Text Placeholder
 * API существует лишь там, и на Forge с NeoForge его типов нет в пути классов
 * вовсе. Отсюда внешний {@code //#if FABRIC==1} — без него не собралась бы
 * половина матрицы.
 *
 * <p>API пережил два разрыва: до 1.19 это {@code PlaceholderAPI.register} в
 * пакете без {@code .api}, с 1.19 — {@code Placeholders.register}, а с 26.x
 * обработчик стал типизированным и метод зовётся {@code registerServer}.
 * {@code PlaceholderResult} при этом один и тот же во всех трёх поколениях,
 * поэтому расходится только регистрация.
 */
final class PlaceholderBridge {

    private static final String NAMESPACE = "noro";

    private PlaceholderBridge() {}

    static void register(ProfileCache profiles) {
        //#if FABRIC==1
        for (String name : PlaceholderValues.ALL_KEYS) {
            //#if MC>=260000
            //$$ Placeholders.registerServer(
            //$$         Identifier.fromNamespaceAndPath(NAMESPACE, name),
            //$$         (context, argument) -> result(profiles, uuid(context.hasPlayer() ? context.player() : null), name, argument));
            //#elseif MC>=12100
            Placeholders.register(
                    ResourceLocation.fromNamespaceAndPath(NAMESPACE, name),
                    (context, argument) -> result(profiles, uuid(context.hasPlayer() ? context.player() : null), name, argument));
            //#elseif MC>=11900
            //$$ Placeholders.register(
            //$$         new ResourceLocation(NAMESPACE, name),
            //$$         (context, argument) -> result(profiles, uuid(context.hasPlayer() ? context.player() : null), name, argument));
            //#else
            //$$ PlaceholderAPI.register(
            //$$         new ResourceLocation(NAMESPACE, name),
            //$$         context -> result(profiles, uuid(context.hasPlayer() ? context.getPlayer() : null), name, context.getArgument()));
            //#endif
        }
        //#endif
    }

    //#if FABRIC==1
    /**
     * @param uuid {@code null}, если плейсхолдер разбирают вне игрока — например
     *             в заголовке таба, который считается один раз на весь сервер
     */
    private static PlaceholderResult result(ProfileCache profiles, UUID uuid, String name, String argument) {
        String value = uuid == null ? null : PlaceholderValues.resolve(profiles.get(uuid), key(name, argument));
        // Пусто, а не invalid: ключи мы регистрируем сами, и отсутствие значения
        // означает «игрока нет в кэше» — так бывает у сообщения о выходе и у
        // строк, которые считаются один раз на весь сервер. invalid вылезал бы
        // в чужом тексте надписью «[unknown placeholder]».
        return PlaceholderResult.value(value == null ? "" : value);
    }

    /** Единственное обращение к Minecraft: {@code getUUID()} есть на всём диапазоне. */
    private static UUID uuid(net.minecraft.world.entity.Entity player) {
        return player == null ? null : player.getUUID();
    }
    //#endif

    /**
     * Аргумент приклеивается к имени: {@code %noro:has_role admin%} и
     * {@code %noro_has_role_admin%} на Paper обязаны означать одно и то же.
     */
    private static String key(String name, String argument) {
        return argument == null || argument.isBlank() ? name : name + "_" + argument.strip();
    }
}
