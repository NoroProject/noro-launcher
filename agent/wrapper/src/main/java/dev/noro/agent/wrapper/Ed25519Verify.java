package dev.noro.agent.wrapper;

import java.security.GeneralSecurityException;
import java.security.KeyFactory;
import java.security.PublicKey;
import java.security.Signature;
import java.security.spec.X509EncodedKeySpec;
import java.util.HexFormat;

/**
 * Проверка ed25519-подписи мастера штатными средствами JDK.
 *
 * <p>С Java 15 ed25519 есть в JCA, поэтому ни BouncyCastle, ни своей реализации
 * кривой не нужно — в врапперe, который раздаёт исполняемый код, чужой крипто-код
 * лишний.
 */
public final class Ed25519Verify {

    /** Заголовок SubjectPublicKeyInfo для ed25519; дальше идут 32 байта ключа. */
    private static final byte[] SPKI_PREFIX = HexFormat.of().parseHex("302a300506032b6570032100");

    private final PublicKey publicKey;

    public Ed25519Verify(String publicKeyHex) {
        byte[] raw = HexFormat.of().parseHex(publicKeyHex.strip());
        if (raw.length != 32) {
            throw new IllegalArgumentException("ed25519 public key must be 32 bytes (64 hex chars)");
        }
        byte[] spki = new byte[SPKI_PREFIX.length + raw.length];
        System.arraycopy(SPKI_PREFIX, 0, spki, 0, SPKI_PREFIX.length);
        System.arraycopy(raw, 0, spki, SPKI_PREFIX.length, raw.length);
        try {
            this.publicKey = KeyFactory.getInstance("Ed25519").generatePublic(new X509EncodedKeySpec(spki));
        } catch (GeneralSecurityException e) {
            throw new IllegalArgumentException("bad ed25519 public key", e);
        }
    }

    public boolean verify(byte[] message, String signatureHex) {
        try {
            Signature signature = Signature.getInstance("Ed25519");
            signature.initVerify(publicKey);
            signature.update(message);
            return signature.verify(HexFormat.of().parseHex(signatureHex.strip()));
        } catch (GeneralSecurityException | IllegalArgumentException e) {
            return false;
        }
    }
}
