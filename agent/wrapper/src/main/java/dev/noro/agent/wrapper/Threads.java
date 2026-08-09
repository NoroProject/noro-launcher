package dev.noro.agent.wrapper;

import java.util.concurrent.ThreadFactory;

/** Потоки враппера — демоны: ни один из них не должен удерживать JVM. */
final class Threads {

    private Threads() {}

    static ThreadFactory daemonFactory(String name) {
        return r -> {
            Thread thread = new Thread(r, name);
            thread.setDaemon(true);
            return thread;
        };
    }
}
