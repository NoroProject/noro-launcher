package dev.noro.agent.wrapper;

import java.io.IOException;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;

/** SHA-1 в hex: мастер адресует файлы им же, и сверять надо тем же алгоритмом. */
final class Sha1 {

    private Sha1() {}

    static String of(byte[] data) throws IOException {
        try {
            return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-1").digest(data));
        } catch (NoSuchAlgorithmException e) {
            throw new IOException("SHA-1 unavailable", e);
        }
    }
}
