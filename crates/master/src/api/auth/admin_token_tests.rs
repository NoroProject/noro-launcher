use super::*;

#[test]
fn a_fresh_token_carries_the_recognisable_prefix() {
    let secret = generate();
    assert!(secret.starts_with(PREFIX));
    // 32 байта в hex — угадать нельзя, а сканеру видно, где кончается префикс.
    assert_eq!(secret.len(), PREFIX.len() + 64);
}

#[test]
fn two_tokens_never_repeat() {
    assert_ne!(generate(), generate());
}

#[test]
fn the_secret_verifies_against_its_own_hash() {
    let secret = generate();
    let phc = hash(&secret).unwrap();
    assert!(verify(&secret, &phc));
}

#[test]
fn another_secret_does_not() {
    let phc = hash(&generate()).unwrap();
    assert!(!verify(&generate(), &phc));
}

#[test]
fn the_stored_hash_is_not_itself_a_working_token() {
    // Ровно то, что чинится этой заменой: раньше содержимое колонки совпадало
    // с тем, с чем сравнивался предъявленный токен.
    let secret = generate();
    let phc = hash(&secret).unwrap();
    assert!(!verify(&phc, &phc));
    assert_ne!(phc, lookup(&secret));
}

#[test]
fn the_same_secret_hashes_differently_every_time() {
    let secret = generate();
    assert_ne!(hash(&secret).unwrap(), hash(&secret).unwrap());
}

#[test]
fn the_selector_is_stable() {
    let secret = generate();
    assert_eq!(lookup(&secret), lookup(&secret));
}

#[test]
fn a_corrupted_hash_rejects_instead_of_panicking() {
    assert!(!verify(&generate(), "не PHC"));
}
