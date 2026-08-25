use super::*;
use axum::body::to_bytes;

async fn body_of(e: AppError) -> serde_json::Value {
    let res = e.into_response();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn отказ_несёт_код_и_чистое_сообщение() {
    let body = body_of(AppError::NotFound("build".into())).await;
    assert_eq!(body["error"]["code"], "not_found");
    // Номер — то, что человек продиктует в поддержку.
    assert_eq!(body["error"]["number"], 1301);
    // Без служебного префикса: вид отказа теперь в code, а не в тексте.
    assert_eq!(body["error"]["message"], "build");
}

#[tokio::test]
async fn уточнённый_код_доезжает_до_клиента() {
    let e = AppError::coded(
        StatusCode::FORBIDDEN,
        crate::error_codes::STEP_UP_REQUIRED,
        "confirm it is you",
    );
    let body = body_of(e).await;
    assert_eq!(body["error"]["code"], "step_up_required");
    assert_eq!(body["error"]["number"], 1003);
}

/// Отказ по форме несёт поля: интерфейсу есть что подсветить, а не только
/// абзац текста под всей формой.
#[tokio::test]
async fn отказ_по_форме_несёт_поля() {
    let e = AppError::Validation(vec![crate::api::validate::FieldError {
        field: "name".into(),
        code: "required",
        message: "this field is required".into(),
    }]);
    let res = e.into_response();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["code"], "validation_error");
    assert_eq!(body["error"]["number"], 1102);
    assert_eq!(body["error"]["details"][0]["field"], "name");
    assert_eq!(body["error"]["details"][0]["code"], "required");
}

/// У остальных отказов `details` нет вовсе: клиенту не приходится отличать
/// пустой список от отсутствующего.
#[tokio::test]
async fn у_обычного_отказа_details_отсутствует() {
    let body = body_of(AppError::NotFound("build".into())).await;
    assert!(body["error"].get("details").is_none());
}

#[tokio::test]
async fn внутренняя_ошибка_не_рассказывает_про_базу() {
    let e = AppError::Other(anyhow::anyhow!("relation \"users\" does not exist"));
    let body = body_of(e).await;
    assert_eq!(body["error"]["code"], "internal_error");
    assert_eq!(body["error"]["number"], 1901);
    assert_eq!(body["error"]["message"], "internal error");
}
