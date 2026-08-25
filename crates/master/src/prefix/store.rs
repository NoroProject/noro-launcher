//! Пак плашек как файл мастера: собрать по ролям и положить в хранилище.

use super::pack;
use crate::state::AppState;
use anyhow::Result;
use serde::Serialize;

/// Готовый пак: где лежит, чем проверить и какой символ у какой роли.
///
/// Отдаётся и агенту, и врапперу: первому — чтобы собрать префикс, второму —
/// чтобы прописать пак в `server.properties`.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Current {
    pub url: String,
    pub sha1: String,
    /// Роль → символ в шрифте `noro:prefix`.
    pub glyphs: Vec<Glyph>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Glyph {
    pub role: String,
    /// Символ строкой: в JSON он и так поедет строкой, а `char` там неудобен.
    pub symbol: String,
}

/// Готовый пак, собранный в прошлый раз.
///
/// Держится в памяти, чтобы агент, спрашивающий состав при каждом входе игрока,
/// не запускал пересборку. Момент пересборки выбирает админ кнопкой: он правит
/// роли пачкой, и рассылать игрокам новый пак после каждой правки — значит
/// заставлять их перезагружать ресурсы по десять раз подряд.
static CURRENT: std::sync::RwLock<Option<Current>> = std::sync::RwLock::new(None);

/// Что отдать агенту: собранное ранее, а если ещё не собирали — собрать сейчас.
pub async fn current(state: &AppState) -> Result<Current> {
    if let Some(had) = CURRENT.read().ok().and_then(|c| c.clone()) {
        return Ok(had);
    }
    rebuild(state).await
}

/// Пересобрать пак по текущим ролям и сохранить его.
///
/// Базовая роль пропускается: плашка «ИГРОК» у всех подряд превращает чат в
/// стену одинаковых значков и перестаёт что-либо значить.
pub async fn rebuild(state: &AppState) -> Result<Current> {
    let roles = crate::db::list_roles(&state.db).await?;
    // Ссылки на свои картинки берём отдельным запросом: модель роли для API их
    // не несёт, а тянуть их туда ради пака значило бы показывать их всем.
    let uploaded: std::collections::HashMap<String, String> =
        sqlx::query_as::<_, (String, String)>(
            "SELECT name, badge_sha1 FROM roles WHERE badge_sha1 IS NOT NULL",
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .collect();

    let mut input: Vec<pack::Role> = Vec::new();
    for role in roles.iter().filter(|r| !r.is_default) {
        let image = match uploaded.get(&role.name) {
            Some(sha1) => tokio::fs::read(state.files.path_for(sha1)).await.ok(),
            None => None,
        };
        input.push(pack::Role {
            name: role.name.clone(),
            text: text_of(role),
            color: role.color.clone().unwrap_or_default(),
            image,
        });
    }

    let built = pack::build(&input)?;
    let stored = state.files.put_bytes(&built.zip).await?;
    attach_to_builds(state, &stored).await;

    let made = Current {
        url: format!("{}/files/{}", state.config.public_url, stored.sha1),
        sha1: stored.sha1,
        glyphs: built
            .glyphs
            .into_iter()
            .map(|(role, symbol)| Glyph {
                role,
                symbol: symbol.to_string(),
            })
            .collect(),
    };
    if let Ok(mut slot) = CURRENT.write() {
        *slot = Some(made.clone());
    }
    Ok(made)
}

/// Положить пак файлом в каждую опубликованную сборку.
///
/// Тем, кто заходит лаунчером, пак приезжает вместе со сборкой и включается
/// сразу: окна «принять ресурспак?» они не видят вовсе. Выдача агентом остаётся
/// для остальных и для случая, когда роли поправили посреди сессии.
///
/// Ошибка здесь не должна ронять сборку пака: без файла в сборке он всё равно
/// доедет выдачей.
async fn attach_to_builds(state: &AppState, stored: &crate::files::StoredFile) {
    let builds: Vec<(uuid::Uuid, uuid::Uuid)> =
        match sqlx::query_as("SELECT id, server_id FROM builds")
            .fetch_all(&state.db)
            .await
        {
            Ok(ids) => ids,
            Err(e) => {
                tracing::warn!(error = %e, "не перечислить сборки для пака плашек");
                return;
            }
        };
    // По одному кадру на сервер, а не на сборку: у сервера их бывает несколько,
    // и лаунчер на каждый кадр перечитывает манифест и заново применяет паки —
    // игрок видел перезагрузку ресурсов дважды подряд.
    let mut told: std::collections::HashSet<uuid::Uuid> = std::collections::HashSet::new();
    for (id, server_id) in builds {
        // Уже такой же — не трогаем: лишняя рассылка «сборка изменилась»
        // заставила бы лаунчеры перепроверять файлы на пустом месте.
        if same_file(state, id, &stored.sha1).await {
            continue;
        }
        if let Err(e) = crate::db::upsert_build_file(
            &state.db,
            id,
            PACK_PATH,
            &stored.sha1,
            stored.size as i64,
            "client",
            "resourcepack",
        )
        .await
        {
            tracing::warn!(error = %e, build = %id, "не приложить пак плашек к сборке");
            continue;
        }
        // Иначе лаунчер узнает о новом паке только при следующей смене версии
        // сборки: манифест он перечитывает по этому кадру.
        if told.insert(server_id) {
            state
                .ws
                .broadcast(&schema::ServerWsMsg::BuildsChanged { server_id });
        }
    }
}

/// Лежит ли в сборке уже ровно этот пак.
async fn same_file(state: &AppState, build: uuid::Uuid, sha1: &str) -> bool {
    sqlx::query_scalar::<_, String>(
        "SELECT sha1 FROM build_files WHERE build_id = $1 AND path = $2",
    )
    .bind(build)
    .bind(PACK_PATH)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .is_some_and(|had| had == sha1)
}

/// Имя файла в сборке. Постоянное: иначе каждая пересборка оставляла бы в папке
/// игрока ещё один пак, и они копились бы до бесконечности.
const PACK_PATH: &str = "resourcepacks/noro-prefixes.zip";

/// Что написать на плашке: заданный админом префикс, иначе название роли.
///
/// Из префикса выбрасываются коды цвета: на плашке цвет задаёт градиент, а
/// «§x§f§f…» в тексте превратился бы в набор непонятных букв.
fn text_of(role: &schema::user::Role) -> String {
    let raw = role
        .prefix
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&role.display_name);
    strip_codes(raw)
}

pub(super) fn strip_codes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '§' | '&' if chars.peek().is_some_and(|n| n.is_ascii_alphanumeric()) => {
                chars.next();
            }
            '#' if chars.clone().take(6).all(|c| c.is_ascii_hexdigit()) => {
                for _ in 0..6 {
                    chars.next();
                }
            }
            _ => out.push(ch),
        }
    }
    out.trim().to_string()
}
