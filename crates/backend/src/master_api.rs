// Over 150 lines: one thin client for every player-facing endpoint the master
// has. Split per area it would be six copies of the same four lines.
//! The player's own data on the master, over HTTP.
//!
//! The socket carries what the master decides to push; this carries what the
//! launcher decides to ask for. Splitting them that way keeps the protocol from
//! growing a request/response pair for every list the interface shows — a feed,
//! a thread of messages and a rule book are pages, and a page is a GET.
//!
//! Every call needs the session token, so it is taken once at construction. A
//! call made without one is a bug in the caller, not a reason to ask the player
//! to sign in again, and it comes back as `Unauthorized`.

use anyhow::{anyhow, Context, Result};
use schema::java::JavaRuntimeOption;
use schema::personal::{InstallRequest, PersonalItem};
use schema::Page;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use uuid::Uuid;

/// What the master offers for this build, and what the player picked.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct JavaRuntimes {
    pub options: Vec<JavaRuntimeOption>,
    pub default_component: String,
    pub selected: Option<String>,
}

pub struct MasterApi {
    base: String,
    token: String,
    http: reqwest::Client,
}

impl MasterApi {
    /// `None` when nobody is signed in: everything here is somebody's own data,
    /// and there is no anonymous version of it to fetch.
    pub fn new(http: reqwest::Client, master_url: &str, token: Option<String>) -> Option<Self> {
        Some(Self {
            base: master_url.trim_end_matches('/').to_string(),
            token: token?,
            http,
        })
    }

    /// The client for the current session, `None` when nobody is signed in.
    pub fn for_session(ctx: &crate::backend::Ctx) -> Option<Self> {
        Self::new(
            ctx.http.clone(),
            &ctx.config.get().master_url,
            ctx.ws.token(),
        )
    }

    /// A request to `path` on the master with the session token. Every call to
    /// the master starts here, including the ones that read the answer their
    /// own way: the in-game case panel turns statuses into its own messages,
    /// and the startup check tells a refused token from a missing network.
    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.http
            .request(method, format!("{}{path}", self.base))
            .bearer_auth(&self.token)
    }

    async fn send<T: DeserializeOwned>(&self, req: reqwest::RequestBuilder) -> Result<T> {
        let res = req.send().await.context("the master is not answering")?;
        parse(res).await
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.send(self.request(reqwest::Method::GET, path)).await
    }

    async fn post<T: DeserializeOwned>(&self, path: &str, body: &Value) -> Result<T> {
        self.send(self.request(reqwest::Method::POST, path).json(body))
            .await
    }

    async fn put<T: DeserializeOwned>(&self, path: &str, body: &Value) -> Result<T> {
        self.send(self.request(reqwest::Method::PUT, path).json(body))
            .await
    }

    async fn delete(&self, path: &str) -> Result<()> {
        self.send::<Value>(self.request(reqwest::Method::DELETE, path))
            .await
            .map(|_| ())
    }

    // ── Notifications ───────────────────────────────────────────────────────

    pub async fn notifications(
        &self,
        offset: u32,
        limit: u32,
        unread_only: bool,
    ) -> Result<Page<schema::notifications::Notification>> {
        self.get(&format!(
            "/api/notifications?offset={offset}&limit={limit}&unread={unread_only}"
        ))
        .await
    }

    pub async fn unread_count(&self) -> Result<i64> {
        let v: Value = self.get("/api/notifications/unread").await?;
        Ok(v.get("unread").and_then(Value::as_i64).unwrap_or(0))
    }

    /// Returns the new unread count — the master already knows it, and a second
    /// request to find out would race with the next notification arriving.
    pub async fn mark_read(&self, id: Uuid) -> Result<i64> {
        let v: Value = self
            .post(&format!("/api/notifications/{id}/read"), &json!({}))
            .await?;
        Ok(v.get("unread").and_then(Value::as_i64).unwrap_or(0))
    }

    pub async fn mark_all_read(&self) -> Result<()> {
        self.post::<Value>("/api/notifications/read-all", &json!({}))
            .await
            .map(|_| ())
    }

    // ── Personal content ────────────────────────────────────────────────────

    pub async fn personal_content(&self, server_id: Uuid) -> Result<Vec<PersonalItem>> {
        self.get(&format!("/api/me/content?server_id={server_id}"))
            .await
    }

    pub async fn install_content(&self, req: &InstallRequest) -> Result<PersonalItem> {
        self.post("/api/me/content", &serde_json::to_value(req)?)
            .await
    }

    pub async fn remove_content(&self, id: Uuid) -> Result<()> {
        self.delete(&format!("/api/me/content/{id}")).await
    }

    pub async fn set_content_enabled(&self, id: Uuid, enabled: bool) -> Result<()> {
        self.post::<Value>(
            &format!("/api/me/content/{id}/enabled"),
            &json!({ "enabled": enabled }),
        )
        .await
        .map(|_| ())
    }

    // ── Catalogue ───────────────────────────────────────────────────────────

    pub async fn catalog_search(&self, query: &str) -> Result<Value> {
        self.get(&format!("/api/catalog/search?{query}")).await
    }

    pub async fn catalog_project(&self, provider: &str, id: &str) -> Result<Value> {
        self.get(&format!(
            "/api/catalog/{}/project/{}",
            urlencoding::encode(provider),
            urlencoding::encode(id)
        ))
        .await
    }

    pub async fn catalog_versions(&self, provider: &str, id: &str, query: &str) -> Result<Value> {
        self.get(&format!(
            "/api/catalog/{}/project/{}/versions?{query}",
            urlencoding::encode(provider),
            urlencoding::encode(id)
        ))
        .await
    }

    // ── Skins and capes ─────────────────────────────────────────────────────

    /// Answers with the updated profile, same as an upload.
    pub async fn set_skin_model(&self, slim: bool) -> Result<schema::UserProfile> {
        self.put(
            "/api/me/skin/model",
            &json!({ "model": if slim { "slim" } else { "classic" } }),
        )
        .await
    }

    pub async fn upload_skin(&self, png: Vec<u8>) -> Result<schema::UserProfile> {
        let part = reqwest::multipart::Part::bytes(png)
            .file_name("skin.png")
            .mime_str("image/png")?;
        self.send(
            self.request(reqwest::Method::POST, "/api/me/skin")
                .multipart(reqwest::multipart::Form::new().part("skin", part)),
        )
        .await
    }

    pub async fn capes(&self) -> Result<Vec<schema::CapeRow>> {
        self.get("/api/capes").await
    }

    pub async fn select_cape(&self, cape_id: Option<Uuid>) -> Result<schema::UserProfile> {
        self.put(
            "/api/me/cape",
            &serde_json::to_value(schema::SelectCapeReq { cape_id })?,
        )
        .await
    }

    pub async fn skin_presets(&self) -> Result<Vec<bridge::ServerSkinPresetItem>> {
        self.get("/api/me/skin-presets").await
    }

    pub async fn suggest_mod(&self, body: &Value) -> Result<()> {
        self.post::<Value>("/api/mod-suggestions", body)
            .await
            .map(|_| ())
    }

    // ── Launcher housekeeping ───────────────────────────────────────────────

    /// Trades a grant the player confirmed for a session in their account.
    pub async fn claim_impersonation(&self, grant_id: Uuid) -> Result<Value> {
        self.post(
            "/api/launcher/impersonate/claim",
            &json!({ "grant_id": grant_id }),
        )
        .await
    }

    /// `query` is the already-encoded query string.
    pub async fn send_support_bundle(&self, query: &str, archive: Vec<u8>) -> Result<Value> {
        self.send(
            self.request(
                reqwest::Method::POST,
                &format!("/api/launcher/support-bundle?{query}"),
            )
            .header("content-type", "application/zip")
            .body(archive),
        )
        .await
    }

    // ── Java runtimes ───────────────────────────────────────────────────────

    pub async fn java_runtimes(&self, server_id: Uuid) -> Result<JavaRuntimes> {
        self.get(&format!(
            "/api/me/java?server_id={server_id}&platform={}",
            schema::current_platform()
        ))
        .await
    }

    /// `None` goes back to the runtime the build ships.
    ///
    /// Slow on a first pick: the master downloads the runtime before it records
    /// the choice, so that a recorded choice always has files behind it.
    pub async fn set_java_runtime(&self, server_id: Uuid, component: Option<&str>) -> Result<()> {
        self.put::<Value>(
            "/api/me/java",
            &json!({
                "server_id": server_id,
                "platform": schema::current_platform(),
                "component": component,
            }),
        )
        .await
        .map(|_| ())
    }

    // ── The rest of the player's own pages ──────────────────────────────────

    pub async fn punishments(&self) -> Result<Value> {
        self.get("/api/me/punishments").await
    }

    pub async fn rules(&self) -> Result<Value> {
        self.get("/api/rules").await
    }

    pub async fn tickets(&self, offset: u32, limit: u32) -> Result<Value> {
        self.get(&format!("/api/tickets?offset={offset}&limit={limit}"))
            .await
    }

    pub async fn ticket(&self, id: Uuid) -> Result<Value> {
        self.get(&format!("/api/tickets/{id}")).await
    }

    pub async fn ticket_reply(&self, id: Uuid, content: &str) -> Result<Value> {
        self.post(
            &format!("/api/tickets/{id}/messages"),
            &json!({ "content": content }),
        )
        .await
    }

    pub async fn open_ticket(&self, subject: &str, content: &str) -> Result<Value> {
        self.post(
            "/api/tickets",
            &json!({ "subject": subject, "content": content }),
        )
        .await
    }

    pub async fn dm_threads(&self) -> Result<Value> {
        self.get("/api/dm").await
    }

    pub async fn dm_thread(&self, peer: Uuid, offset: u32, limit: u32) -> Result<Value> {
        self.get(&format!("/api/dm/{peer}?offset={offset}&limit={limit}"))
            .await
    }

    pub async fn dm_send(&self, peer: Uuid, body: &str) -> Result<Value> {
        self.post(&format!("/api/dm/{peer}"), &json!({ "body": body }))
            .await
    }

    pub async fn dm_mark_read(&self, peer: Uuid) -> Result<()> {
        self.post::<Value>(&format!("/api/dm/{peer}/read"), &json!({}))
            .await
            .map(|_| ())
    }
}

/// The master's refusals carry a message; keeping it is the difference between
/// "could not install" and "staff blocked this mod".
async fn parse<T: DeserializeOwned>(res: reqwest::Response) -> Result<T> {
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        let detail = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|v| {
                v.get("error")
                    .or_else(|| v.get("message"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_else(|| body.chars().take(200).collect());
        return Err(anyhow!("{status}: {detail}"));
    }
    // An empty success body reads as `null`, so a call that only cares about
    // the status doesn't fail on a 204.
    let body = if body.trim().is_empty() {
        "null"
    } else {
        body.as_str()
    };
    serde_json::from_str(body).context("the master answered with something unexpected")
}

#[cfg(test)]
#[path = "master_api_tests.rs"]
mod tests;
