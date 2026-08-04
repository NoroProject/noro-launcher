//! Раздача jar'ов агента враппера.
//!
//! Подпись покрывает весь дескриптор, а он содержит sha1 — значит подтверждает
//! и содержимое файла. Врапперу остаётся сверить хеш скачанного, и подмена
//! jar'а по дороге становится невозможной. Это защита канала доставки, которая
//! работает по-настоящему — в отличие от попыток проверить клиент игрока.

use crate::api::auth::AgentAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ArtifactQuery {
    /// `paper`, `fabric` или `neoforge`.
    pub platform: String,
    /// Версия Minecraft, например `1.21.1`.
    pub mc: String,
}

/// Порядок полей — часть контракта: враппер собирает те же байты для проверки
/// подписи, а serde сериализует поля в порядке объявления.
#[derive(Serialize, Clone)]
pub struct AgentArtifact {
    pub platform: String,
    pub mc_version: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
    /// ed25519 в hex. При подписи поле пустое — так же, как в манифестах сборок.
    pub signature: String,
}

impl AgentArtifact {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let unsigned = Self {
            signature: String::new(),
            ..self.clone()
        };
        serde_json::to_vec(&unsigned).expect("AgentArtifact всегда сериализуется")
    }
}

pub async fn artifact(
    State(state): State<AppState>,
    _agent: AgentAuth,
    Query(query): Query<ArtifactQuery>,
) -> AppResult<Json<AgentArtifact>> {
    let platform = safe_segment(&query.platform)?;
    let mc_version = safe_segment(&query.mc)?;

    // Админ кладёт собранные jar'ы в {data_dir}/agents. Отдельная таблица здесь
    // ничего не добавила бы: имя файла и есть весь ключ поиска.
    let path = state
        .config
        .data_dir
        .join("agents")
        .join(format!("{platform}-{mc_version}.jar"));
    if !path.is_file() {
        return Err(AppError::NotFound(format!(
            "агент для {platform} {mc_version} не собран"
        )));
    }

    // put_file не перезаписывает уже лежащее — повторный запрос стоит одного
    // чтения и sha1, а раздачей займётся обычный /files/{sha1} с Range и ETag.
    let stored = state.files.put_file(&path).await?;
    let mut artifact = AgentArtifact {
        platform,
        mc_version,
        url: state.config.file_url(&stored.sha1),
        sha1: stored.sha1,
        size: stored.size,
        signature: String::new(),
    };
    artifact.signature = hex::encode(state.signer.sign(&artifact.signing_bytes()));
    Ok(Json(artifact))
}

/// Публичный ключ подписи — чтобы враппер не заставлял админа переносить его
/// руками на каждый игровой сервер и не расходился между ними.
///
/// Ключ не секрет, но и не якорь доверия сам по себе: взятый по тому же каналу,
/// что и артефакт, он не защитит от подмены этого канала. Поэтому враппер
/// запоминает его при первом запуске и дальше сверяет только с запомненным.
pub async fn pubkey(
    State(state): State<AppState>,
    _agent: AgentAuth,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "public_key": state.signer.public_key_hex(),
    })))
}

/// Оба параметра идут в имя файла, поэтому путь через них проложить нельзя.
fn safe_segment(raw: &str) -> AppResult<String> {
    let ok = !raw.is_empty()
        && raw.len() <= 32
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_');
    if !ok {
        return Err(AppError::BadRequest(format!("недопустимое значение: {raw}")));
    }
    Ok(raw.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Байты подписи — межъязыковой контракт: ровно эту строку собирает и
    /// проверяет ArtifactDescriptor во враппере. Тест держит обе стороны вместе.
    #[test]
    fn signing_bytes_are_stable() {
        let artifact = AgentArtifact {
            platform: "paper".into(),
            mc_version: "1.21.1".into(),
            sha1: "da39a3ee5e6b4b0d3255bfef95601890afd80709".into(),
            size: 1234,
            url: "http://localhost:8080/files/da39a3ee5e6b4b0d3255bfef95601890afd80709".into(),
            signature: "не должно попасть в подпись".into(),
        };
        let expected = concat!(
            r#"{"platform":"paper","mc_version":"1.21.1","#,
            r#""sha1":"da39a3ee5e6b4b0d3255bfef95601890afd80709","size":1234,"#,
            r#""url":"http://localhost:8080/files/da39a3ee5e6b4b0d3255bfef95601890afd80709","#,
            r#""signature":""}"#
        );
        assert_eq!(String::from_utf8(artifact.signing_bytes()).unwrap(), expected);
    }
}
