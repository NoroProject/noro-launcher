package dev.noro.agent.core;

/**
 * Единственное, что heartbeat'у нужно знать про конкретную платформу.
 *
 * <p>Paper, NeoForge и Fabric считают онлайн по-разному, поэтому цифры приходят
 * снаружи, а вся работа с сетью и расписанием остаётся в core.
 */
public interface ServerStatus {

    int online();

    int maxPlayers();

    /** Ядро и его версия, например {@code "Paper 1.21.1"} — видно в админке. */
    String version();
}
