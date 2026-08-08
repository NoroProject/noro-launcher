//! Генерация пары ключей подписи.
//!
//! Запуск: `cargo run -p master --example keygen`
//!
//! Вынесено в пример, а не в bin: утилита нужна раз в жизнь ключа и не должна
//! попадать в образ мастера. Приватная часть — 32-байтный seed, из которого
//! `Signer25519::from_config` восстанавливает ключ; публичная зашивается в
//! лаунчер и враппер, поэтому обе печатаются вместе — чтобы их нельзя было
//! перепутать или взять из разных запусков.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;

fn main() {
    // OsRng — системный CSPRNG. Любые 32 случайных байта являются валидным
    // seed'ом ed25519, отдельной проверки не требуется.
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);

    let key = SigningKey::from_bytes(&seed);
    let public = hex::encode(key.verifying_key().to_bytes());

    // Пара проверяется здесь же: рассинхрон приватной и публичной части
    // выяснился бы иначе только у игрока, которому лаунчер отказался бы
    // запускаться, — и уже после релиза.
    self_check(&key, &public);

    println!("NORO_SIGNING_KEY={}", hex::encode(seed));
    println!("NORO_SIGNING_PUBKEY={public}");
    println!();
    println!("Приватная часть (NORO_SIGNING_KEY) — только в .env мастера.");
    println!("Публичная (NORO_SIGNING_PUBKEY) — в переменные репозитория GitHub");
    println!("и в signing-public-key у враппера. Смена ключа делает все ранее");
    println!("подписанные сборки лаунчера невалидными.");
}

/// Подписывает пробу приватной частью и проверяет её публичной — ровно так, как
/// это делают мастер и bootstrapper.
fn self_check(key: &SigningKey, public_hex: &str) {
    let probe = b"noro keygen self-check";
    let sig = key.sign(probe);

    let raw = hex::decode(public_hex).expect("публичный ключ должен быть hex");
    let arr: [u8; 32] = raw.try_into().expect("публичный ключ должен быть 32 байта");
    let verifying = VerifyingKey::from_bytes(&arr).expect("публичный ключ должен быть валиден");

    verifying
        .verify(probe, &Signature::from_bytes(&sig.to_bytes()))
        .expect("пара ключей не сходится — публиковать её нельзя");
    assert!(
        verifying
            .verify(b"tampered", &Signature::from_bytes(&sig.to_bytes()))
            .is_err(),
        "подпись обязана отвергать изменённые данные"
    );
}
