//! Admin-API calls to the master on the moderator's behalf.
//!
//! The session token lives here and nowhere else. It never reaches the game's
//! JVM: any mod in the pack could read the environment, the arguments or a file
//! and walk away with full admin access.

use crate::backend::Ctx;
use mod_link::{CaseBrief, CaseView, Dossier};
use schema::Page;
use serde::Deserialize;
use uuid::Uuid;

/// Why it didn't work. Carries the mod's translation keys rather than text —
/// the wording lives in its `lang` files.
///
/// The split is finer than it looks like it needs to be, and each variant earns
/// its place by what the moderator sees: 409 has to read as "another moderator
/// already took this", and 400/422 as "the reason you typed is too short",
/// neither of which should suggest trying again later.
#[derive(Debug, Clone)]
pub enum Denied {
    Forbidden(u16),
    NotFound(u16),
    Conflict(u16),
    Invalid(u16),
    /// The master couldn't be reached, or answered with something unexpected.
    Offline(String),
}

impl Denied {
    /// Number from the master's error registry. `0` when the refusal is ours and
    /// has no number.
    pub fn number(&self) -> u16 {
        match self {
            Denied::Forbidden(n)
            | Denied::NotFound(n)
            | Denied::Conflict(n)
            | Denied::Invalid(n) => *n,
            Denied::Offline(_) => 0,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            Denied::Forbidden(_) => "noro.case.error.forbidden",
            Denied::NotFound(_) => "noro.case.error.gone",
            Denied::Conflict(_) => "noro.case.error.conflict",
            Denied::Invalid(_) => "noro.case.error.invalid",
            Denied::Offline(_) => "noro.case.error.offline",
        }
    }
}

pub type Answer<T> = std::result::Result<T, Denied>;

pub struct Api {
    base: String,
    token: String,
    http: reqwest::Client,
}

impl Api {
    /// `None` when the launcher isn't logged in — nothing to ask the master.
    pub fn new(ctx: &Ctx) -> Option<Self> {
        let token = ctx.ws.token()?;
        Some(Self {
            base: ctx
                .config
                .get()
                .master_url
                .trim_end_matches('/')
                .to_string(),
            token,
            http: ctx.http.clone(),
        })
    }

    /// One page of open cases for the in-game panel.
    ///
    /// The search goes to the master rather than filtering locally: the panel
    /// holds ten cases at a time, and filtering those ten would never find
    /// anything outside them.
    pub async fn queue(&self, query: Option<&str>, offset: i64) -> Answer<Page<CaseBrief>> {
        let mut path = format!(
            "/api/admin/cases?open_only=true&limit={}&offset={}",
            mod_link::QUEUE_PAGE,
            offset.max(0)
        );
        if let Some(q) = query.map(str::trim).filter(|q| !q.is_empty()) {
            path.push_str("&q=");
            path.push_str(&urlencoding::encode(q));
        }
        self.json(self.get(&path)).await
    }

    pub async fn card(&self, case_id: Uuid) -> Answer<CaseView> {
        self.json(self.get(&format!("/api/admin/cases/{case_id}")))
            .await
    }

    pub async fn dossier(&self, username: &str) -> Answer<Dossier> {
        let path = format!(
            "/api/admin/cases/dossier?username={}",
            urlencoding::encode(username)
        );
        self.json(self.get(&path)).await
    }

    /// A public endpoint; the token isn't needed but doesn't hurt.
    pub async fn rules(&self) -> Answer<RulesResponse> {
        self.json(self.get("/api/rules")).await
    }

    /// Account endpoint: returns the punishments of whoever owns the token, and
    /// nobody else's.
    pub async fn own_punishments(&self) -> Answer<Vec<mod_link::OwnPunishment>> {
        self.json(self.get("/api/me/punishments")).await
    }

    pub async fn post(&self, path: &str, body: serde_json::Value) -> Answer<()> {
        self.ok(self.req(reqwest::Method::POST, path).json(&body))
            .await
    }

    pub async fn put(&self, path: &str, body: serde_json::Value) -> Answer<()> {
        self.ok(self.req(reqwest::Method::PUT, path).json(&body))
            .await
    }

    /// A screenshot as an attachment. Same multipart shape the web panel uses.
    pub async fn attach(&self, case_id: Uuid, png: Vec<u8>, note: String) -> Answer<()> {
        let form = reqwest::multipart::Form::new().text("note", note).part(
            "file",
            reqwest::multipart::Part::bytes(png)
                .file_name("shot.png")
                .mime_str("image/png")
                .map_err(|e| Denied::Offline(e.to_string()))?,
        );
        let path = format!("/api/admin/cases/{case_id}/attachments");
        self.ok(self.req(reqwest::Method::POST, &path).multipart(form))
            .await
    }

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.req(reqwest::Method::GET, path)
    }

    fn req(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.http
            .request(method, format!("{}{path}", self.base))
            .bearer_auth(&self.token)
    }

    async fn ok(&self, req: reqwest::RequestBuilder) -> Answer<()> {
        check(req).await.map(|_| ())
    }

    async fn json<T: serde::de::DeserializeOwned>(
        &self,
        req: reqwest::RequestBuilder,
    ) -> Answer<T> {
        let body = check(req).await?;
        serde_json::from_slice(&body).map_err(|e| Denied::Offline(e.to_string()))
    }
}

async fn number_of(res: reqwest::Response) -> u16 {
    number_in(&res.text().await.unwrap_or_default())
}

/// Expects `{"error": {"number": …}}`; anything else is `0`.
fn number_in(body: &str) -> u16 {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("error")?.get("number")?.as_u64())
        .unwrap_or(0) as u16
}

async fn check(req: reqwest::RequestBuilder) -> Answer<Vec<u8>> {
    let res = req
        .send()
        .await
        .map_err(|e| Denied::Offline(e.to_string()))?;
    let status = res.status().as_u16();
    match status {
        200..=299 => res
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| Denied::Offline(e.to_string())),
        401 | 403 => Err(Denied::Forbidden(number_of(res).await)),
        404 => Err(Denied::NotFound(number_of(res).await)),
        409 => Err(Denied::Conflict(number_of(res).await)),
        400 | 422 => {
            // The mod only gets a translation key, so what the master actually
            // objected to has to land in the launcher's log or it's lost.
            let body = res.text().await.unwrap_or_default();
            tracing::debug!(%status, %body, "mod_link: master rejected the form");
            Err(Denied::Invalid(number_in(&body)))
        }
        code => Err(Denied::Offline(format!("HTTP {code}"))),
    }
}

/// Parsed here rather than in the mod: the launcher owns the endpoint's shape
/// and is what gets updated alongside the master.
#[derive(Deserialize)]
pub struct RulesResponse {
    #[serde(default)]
    pub categories: Vec<mod_link::RuleCategory>,
    #[serde(default)]
    pub rules: Vec<mod_link::RuleItem>,
    #[serde(default)]
    pub sanctions: Vec<mod_link::RuleSanction>,
}
