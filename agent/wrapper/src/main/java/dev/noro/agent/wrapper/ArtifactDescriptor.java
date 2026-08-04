package dev.noro.agent.wrapper;

import com.google.gson.FieldNamingPolicy;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import java.nio.charset.StandardCharsets;

/**
 * Ответ {@code GET /api/agent/artifact}.
 *
 * <p>Порядок полей здесь не косметика: подпись считается по JSON с пустым
 * {@code signature}, и байты должны совпасть с теми, что собрал мастер. Serde
 * сериализует поля в порядке объявления — Gson делает то же с компонентами
 * record'а, поэтому порядок обязан повторять структуру на стороне мастера.
 */
public record ArtifactDescriptor(
        String platform, String mcVersion, String sha1, long size, String url, String signature) {

    /** HTML-экранирование Gson включено по умолчанию, а serde его не делает. */
    static final Gson GSON = new GsonBuilder()
            .setFieldNamingPolicy(FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
            .disableHtmlEscaping()
            .create();

    /** Байты, которые подписал мастер: тот же объект с пустой подписью. */
    public byte[] signingBytes() {
        ArtifactDescriptor unsigned = new ArtifactDescriptor(platform, mcVersion, sha1, size, url, "");
        return GSON.toJson(unsigned).getBytes(StandardCharsets.UTF_8);
    }
}
