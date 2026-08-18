//! Загрузка опционального мода файлом.
//!
//! Раньше опциональный мод можно было собрать только из уже лежащих в сборке
//! файлов: сперва залей jar, потом опиши мод отдельной формой. Здесь это один
//! шаг — файл и его описание приезжают вместе.
//!
//! Файл кладётся в `build_files` наравне с остальными, а не рядом: иначе он
//! выпал бы из `verified_files`, и сверка целостности удаляла бы его у игроков
//! как лишний на каждом запуске.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
use axum::Json;
use schema::build::OptionalMod;
use schema::PERM_BUILDS_EDIT;
use serde_json::{json, Value};
use uuid::Uuid;

/// Что пришло из формы. Всё, кроме файла и имени, необязательно.
#[derive(Default)]
struct Form {
    name: String,
    description: String,
    category: String,
    /// `windows,macos,linux` — пусто значит «на всех системах».
    os: Vec<String>,
    conflicts: Vec<String>,
    dependencies: Vec<String>,
    limited: bool,
    enabled_by_default: bool,
    file: Option<bytes::Bytes>,
    filename: Option<String>,
}

/// `POST /api/admin/builds/{id}/optional-mods`
pub async fn upload(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BUILDS_EDIT)?;
    let form = read_form(multipart).await?;

    let data = form
        .file
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("attach the mod file".into()))?;
    let filename = form
        .filename
        .clone()
        .ok_or_else(|| AppError::BadRequest("the file has no name".into()))?;
    let name = form.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("name the mod".into()));
    }

    let path = format!("mods/{filename}");
    let stored = state.files.put_bytes(data).await.map_err(AppError::Other)?;
    crate::db::upsert_build_file(
        &state.db,
        id,
        &path,
        &stored.sha1,
        stored.size as i64,
        "both",
        "mod",
    )
    .await?;

    let mods = add_mod(&state, id, &form, name, &path).await?;
    Ok(Json(
        json!({ "path": path, "sha1": stored.sha1, "optional_mods": mods }),
    ))
}

/// Дописать мод в список сборки. Мод с тем же именем заменяется: повторная
/// загрузка — это обновление файла, а не второй мод-двойник.
async fn add_mod(
    state: &AppState,
    build_id: Uuid,
    form: &Form,
    name: &str,
    path: &str,
) -> AppResult<usize> {
    let build = crate::db::get_build(&state.db, build_id)
        .await?
        .ok_or_else(|| AppError::NotFound("no such build".into()))?;
    let mut mods: Vec<OptionalMod> =
        serde_json::from_value(build.optional_mods.clone()).unwrap_or_default();

    let entry = OptionalMod {
        name: name.to_string(),
        description: form.description.clone(),
        category: if form.category.is_empty() {
            "Прочее".into()
        } else {
            form.category.clone()
        },
        files: vec![path.to_string()],
        enabled_by_default: form.enabled_by_default,
        visible: true,
        limited: form.limited,
        dependencies: form.dependencies.clone(),
        conflicts: form.conflicts.clone(),
        triggers: Vec::new(),
        os: form.os.clone(),
        icon_url: None,
        author: None,
    };
    match mods.iter_mut().find(|m| m.name == name) {
        // Файлы прежней версии оставляем: мод мог состоять из нескольких, а
        // затирать чужие пути загрузкой одного jar нельзя.
        Some(existing) => {
            if !existing.files.iter().any(|f| f == path) {
                existing.files.push(path.to_string());
            }
            existing.description = entry.description;
            existing.category = entry.category;
            existing.limited = entry.limited;
            existing.dependencies = entry.dependencies;
            existing.conflicts = entry.conflicts;
            existing.os = entry.os;
        }
        None => mods.push(entry),
    }

    let value = serde_json::to_value(&mods).map_err(|e| AppError::Other(e.into()))?;
    sqlx::query("UPDATE builds SET optional_mods = $2 WHERE id = $1")
        .bind(build_id)
        .bind(&value)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    Ok(mods.len())
}

async fn read_form(mut multipart: Multipart) -> AppResult<Form> {
    let mut form = Form::default();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" {
            form.filename = field.file_name().map(String::from);
            form.file = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?,
            );
            continue;
        }
        let text = field
            .text()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        match field_name.as_str() {
            "name" => form.name = text,
            "description" => form.description = text,
            "category" => form.category = text,
            "os" => form.os = list(&text),
            "conflicts" => form.conflicts = list(&text),
            "dependencies" => form.dependencies = list(&text),
            "limited" => form.limited = flag(&text),
            "enabled_by_default" => form.enabled_by_default = flag(&text),
            _ => {}
        }
    }
    Ok(form)
}

/// Списки приходят строкой через запятую: форма проще, а разбор один.
fn list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn flag(text: &str) -> bool {
    matches!(text.trim(), "true" | "1" | "on")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_splits_and_trims() {
        assert_eq!(list(" Windows, macOS "), vec!["windows", "macos"]);
        assert!(list("").is_empty());
    }

    /// Галочка в форме приезжает разными словами в зависимости от клиента.
    #[test]
    fn flag_reads_every_form_of_yes() {
        assert!(flag("true") && flag("1") && flag("on"));
        assert!(!flag("false") && !flag(""));
    }
}
