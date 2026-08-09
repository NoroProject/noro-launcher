package dev.noro.agent.wrapper;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * Исполнение операций, пришедших от мастера.
 *
 * <p>Одна точка входа вместо развилки внутри канала: канал занимается сокетом и
 * ничего не знает про сервер, а здесь наоборот.
 */
public final class ControlOps {

    private final Supervisor supervisor;
    private final ServerFiles files;
    private final ServerMods mods;
    private final ServerBackups backups;

    public ControlOps(Supervisor supervisor, ServerFiles files, ServerMods mods, ServerBackups backups) {
        this.supervisor = supervisor;
        this.files = files;
        this.mods = mods;
        this.backups = backups;
    }

    /** @return данные ответа; {@code null} — операция без результата */
    public JsonElement execute(String op, JsonObject args) throws Exception {
        return switch (op) {
            case "power" -> power(str(args, "action"));
            case "command" -> {
                supervisor.command(str(args, "line"));
                yield null;
            }
            case "fs_list" -> files.list(str(args, "path"));
            case "fs_read" -> files.read(str(args, "path"));
            case "fs_write" -> files.write(str(args, "path"), str(args, "content"));
            case "fs_delete" -> files.delete(str(args, "path"));
            case "fs_mkdir" -> files.mkdir(str(args, "path"));
            case "mod_install" -> mods.install(
                    str(args, "url"), str(args, "sha1"), str(args, "filename"), str(args, "dir"));
            case "backup_create" -> backups.create(str(args, "name"));
            case "backup_list" -> backups.list();
            case "backup_restore" -> backups.restore(str(args, "name"));
            case "backup_delete" -> backups.delete(str(args, "name"));
            default -> throw new IllegalArgumentException("unsupported operation: " + op);
        };
    }

    private JsonElement power(String action) throws Exception {
        switch (action) {
            case "start" -> supervisor.start();
            case "stop" -> supervisor.stop();
            case "restart" -> supervisor.restart();
            case "kill" -> supervisor.kill();
            default -> throw new IllegalArgumentException("unknown power action: " + action);
        }
        return null;
    }

    static String str(JsonObject args, String key) {
        if (!args.has(key) || args.get(key).isJsonNull()) {
            throw new IllegalArgumentException("missing argument: " + key);
        }
        return args.get(key).getAsString();
    }
}
