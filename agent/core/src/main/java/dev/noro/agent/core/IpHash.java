package dev.noro.agent.core;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;

/**
 * Адрес игрока в виде, который можно хранить.
 *
 * <p>Сырой IP мастеру не нужен ни для чего: все задачи, ради которых его
 * вообще спрашивают — «этот же человек заходил под другим ником», «бан обошли с
 * того же адреса», — решаются сравнением, а сравнивать можно и хеши. Хранить
 * при этом нечего, что стоило бы утечки.
 *
 * <p>Соль общая и постоянная, из секрета сервера: без неё диапазон домашних
 * адресов перебирается целиком за минуты, а с ней хеш бесполезен снаружи.
 */
public final class IpHash {

    private IpHash() {}

    /** @return шестнадцатеричный хеш либо {@code null}, если адреса нет */
    public static String of(String address, String salt) {
        if (address == null || address.isBlank()) {
            return null;
        }
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            digest.update(salt.getBytes(StandardCharsets.UTF_8));
            byte[] hash = digest.digest(address.getBytes(StandardCharsets.UTF_8));
            StringBuilder hex = new StringBuilder(32);
            // Половины хватает: коллизия на 64 битах требует миллиардов адресов,
            // а строка вдвое короче лежит в логах и в базе.
            for (int i = 0; i < 8; i++) {
                hex.append(String.format("%02x", hash[i]));
            }
            return hex.toString();
        } catch (NoSuchAlgorithmException e) {
            // SHA-256 обязателен для любой JVM — сюда не попасть.
            throw new IllegalStateException(e);
        }
    }
}
