package dev.noro.client.player;

import com.google.gson.JsonObject;
import dev.noro.client.link.Feature;
import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.Component;

/**
 * Перезагрузка ресурсов по просьбе лаунчера.
 *
 * <p>Живёт в моде игрока, а не в ядре: ядро — это канал до лаунчера и общая
 * библиотека экранов, а перезагрузка наборов уже поведение, и касается оно
 * каждого игрока, а не только модератора.
 *
 * <p>Лаунчер подменяет наборы прямо под работающей игрой, но игра держит в
 * памяти прежние: чтобы новое стало видно, нужна перезагрузка. Снаружи процесса
 * такой ручки нет, поэтому зовёт её мод.
 *
 * <p>Экран загрузки при этом будет — он и есть сама пересборка атласов, моделей
 * и шрифтов, а не заставка поверх неё. Зато случается он один раз и по делу, а
 * не при каждом входе на сервер.
 */
public final class Resources implements Feature {

    @Override
    public String id() {
        return "resources";
    }

    @Override
    public boolean accept(String type, JsonObject envelope) {
        // Имя варианта как есть, без нижних подчёркиваний: конверт помечается
        // `#[serde(tag = "type")]` без переименования, и остальные кадры тоже
        // приходят так — «Rules», «Queue».
        if (!"ReloadResources".equals(type)) {
            return false;
        }
        Minecraft mc = Minecraft.getInstance();
        mc.execute(() -> {
            // Перед экраном загрузки говорим, что происходит: иначе он выглядит
            // как беспричинный фриз посреди игры.
            if (mc.player != null) {
                mc.player.displayClientMessage(
                        Component.translatable("noro.player.resources.reloading"), true);
            }
            mc.reloadResourcePacks();
        });
        return true;
    }
}
