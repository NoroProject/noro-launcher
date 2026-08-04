package dev.noro.agent.wrapper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.junit.jupiter.api.Test;

/**
 * Межъязыковой контракт подписи.
 *
 * <p>Мастер подписывает JSON, который собирает serde, а проверяет подпись Java —
 * разойтись они могут на одном лишнем пробеле. Парная сторона этого теста живёт
 * в {@code crates/master/src/api/agent_artifact.rs}: там та же строка зафиксирована
 * как ожидаемый результат {@code signing_bytes()}.
 */
class ArtifactSignatureTest {

    /** Публичный ключ dev-seed из schema::DEV_SIGNING_SEED. */
    private static final String DEV_PUBLIC_KEY =
            "735fcfcd88e76de41d49ab3b6e42ac506ddf213359425b047a05d4c4a77f0715";

    private static final String CANONICAL =
            "{\"platform\":\"paper\",\"mc_version\":\"1.21.1\","
                    + "\"sha1\":\"da39a3ee5e6b4b0d3255bfef95601890afd80709\",\"size\":1234,"
                    + "\"url\":\"http://localhost:8080/files/da39a3ee5e6b4b0d3255bfef95601890afd80709\","
                    + "\"signature\":\"\"}";

    /** Настоящая ed25519-подпись CANONICAL тем самым dev-ключом. */
    private static final String SIGNATURE =
            "fcc7859127649882e408af2f1e5bd9fa8e1dc5a92dc707d0edf8fa2768de75f3"
                    + "511fcc065135ef2e85b641cb2f50b49d6d9fec6ea949b1d66e2c884b28e6f400";

    private static ArtifactDescriptor sample(String signature) {
        return new ArtifactDescriptor(
                "paper",
                "1.21.1",
                "da39a3ee5e6b4b0d3255bfef95601890afd80709",
                1234,
                "http://localhost:8080/files/da39a3ee5e6b4b0d3255bfef95601890afd80709",
                signature);
    }

    @Test
    void signingBytesMatchTheMaster() {
        assertEquals(CANONICAL, new String(sample(SIGNATURE).signingBytes(), StandardCharsets.UTF_8));
    }

    @Test
    void acceptsRealMasterSignature() {
        ArtifactDescriptor descriptor = sample(SIGNATURE);
        assertTrue(new Ed25519Verify(DEV_PUBLIC_KEY).verify(descriptor.signingBytes(), descriptor.signature()));
    }

    @Test
    void rejectsTamperedDescriptor() {
        // Подменённый sha1 — ровно та атака, ради которой подпись и нужна:
        // дескриптор увёл бы враппер на чужой jar.
        ArtifactDescriptor tampered = new ArtifactDescriptor(
                "paper",
                "1.21.1",
                "0000000000000000000000000000000000000000",
                1234,
                "http://localhost:8080/files/da39a3ee5e6b4b0d3255bfef95601890afd80709",
                SIGNATURE);
        assertFalse(new Ed25519Verify(DEV_PUBLIC_KEY).verify(tampered.signingBytes(), tampered.signature()));
    }

    @Test
    void rejectsSignatureFromAnotherKey() {
        String otherKey = "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29";
        assertFalse(new Ed25519Verify(otherKey).verify(sample(SIGNATURE).signingBytes(), SIGNATURE));
    }
}
