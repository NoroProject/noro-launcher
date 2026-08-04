package dev.noro.agent.core;

import java.util.UUID;
import java.util.concurrent.CompletableFuture;

/**
 * Раскладка ролей игрока — без единого упоминания типов LuckPerms.
 *
 * <p>Нужен именно интерфейс: как только тип LuckPerms попадает в сигнатуру или
 * в лямбду вызывающего кода, JVM грузит его при резолве этого места, ещё до
 * входа в любой `try`. Поймать `NoClassDefFoundError` там уже нельзя, и агент
 * валит сервер, на котором LuckPerms просто не установлен.
 */
public interface RoleApplier {

    CompletableFuture<Void> apply(UUID mcUuid, PlayerProfile profile);
}
