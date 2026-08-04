package dev.noro.agent.wrapper;

import java.nio.file.Path;
import java.util.Locale;

/** Платформа игрового сервера и каталог, куда ей ставится агент. */
public enum Platform {
    PAPER("plugins"),
    FABRIC("mods"),
    NEOFORGE("mods");

    private final String installDir;

    Platform(String installDir) {
        this.installDir = installDir;
    }

    /** Имя, под которым платформа известна мастеру в {@code ?platform=}. */
    public String id() {
        return name().toLowerCase(Locale.ROOT);
    }

    public Path installDir(Path serverDir) {
        return serverDir.resolve(installDir);
    }

    public static Platform of(String raw) {
        return valueOf(raw.strip().toUpperCase(Locale.ROOT));
    }
}
