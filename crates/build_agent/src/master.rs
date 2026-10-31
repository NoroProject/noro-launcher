//! The master's local-build API, as the agent and `submit` see it.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct Master {
    base: String,
    token: String,
    http: reqwest::Client,
}

#[derive(Deserialize)]
pub struct Claim {
    pub job_id: String,
    pub target: String,
    pub tag: String,
    pub source_url: String,
}

/// What a log post said about the target.
pub enum Beat {
    Kept,
    /// The master gave the target to someone else or the job was cancelled.
    Lost,
}

impl Master {
    pub fn from_env() -> Result<Self> {
        let base = std::env::var("NORO_MASTER_URL").context("NORO_MASTER_URL is not set")?;
        let token = std::env::var("NORO_ADMIN_TOKEN").context("NORO_ADMIN_TOKEN is not set")?;
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            // Uploading the macOS bundle makes the master build and notarise a
            // DMG before it answers; that has taken longer than ten minutes.
            .timeout(Duration::from_secs(40 * 60))
            .build()?;
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            token,
            http,
        })
    }

    fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.http
            .post(format!("{}{path}", self.base))
            .bearer_auth(&self.token)
    }

    async fn ok(resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            bail!("master answered {status}: {body}");
        }
        Ok(serde_json::from_str(&body).unwrap_or(Value::Null))
    }

    pub async fn claim(
        &self,
        agent: &str,
        targets: &[String],
        host: &str,
    ) -> Result<Option<Claim>> {
        let body = json!({
            "agent": agent,
            "targets": targets,
            "host": host,
            "version": env!("CARGO_PKG_VERSION"),
        });
        let v = Self::ok(
            self.post("/api/admin/launcher/local/claim")
                .json(&body)
                .send()
                .await?,
        )
        .await?;
        if v.is_null() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_value(v)?))
    }

    fn part(&self, c: &Claim, agent: &str, what: &str) -> reqwest::RequestBuilder {
        self.post(&format!(
            "/api/admin/launcher/local/jobs/{}/{}/{what}?agent={agent}",
            c.job_id, c.target
        ))
    }

    /// An empty `text` is a bare heartbeat.
    pub async fn log(&self, c: &Claim, agent: &str, text: String) -> Result<Beat> {
        let resp = self.part(c, agent, "log").body(text).send().await?;
        if resp.status() == reqwest::StatusCode::CONFLICT {
            return Ok(Beat::Lost);
        }
        Self::ok(resp).await?;
        Ok(Beat::Kept)
    }

    pub async fn fail(&self, c: &Claim, agent: &str, error: &str) -> Result<()> {
        let req = self.part(c, agent, "fail").json(&json!({ "error": error }));
        Self::ok(req.send().await?).await.map(drop)
    }

    pub async fn upload(&self, c: &Claim, agent: &str, files: &[PathBuf]) -> Result<Value> {
        let mut form = reqwest::multipart::Form::new();
        for path in files {
            let name = file_name(path);
            let bytes = tokio::fs::read(path)
                .await
                .with_context(|| format!("reading {}", path.display()))?;
            form = form.part(
                "file",
                reqwest::multipart::Part::bytes(bytes).file_name(name),
            );
        }
        Self::ok(
            self.part(c, agent, "artifacts")
                .multipart(form)
                .send()
                .await?,
        )
        .await
    }

    pub async fn download(&self, url: &str, dest: &Path) -> Result<()> {
        let resp = self.http.get(url).send().await?.error_for_status()?;
        let bytes = resp.bytes().await?;
        tokio::fs::write(dest, &bytes).await?;
        Ok(())
    }

    pub async fn submit(
        &self,
        tag: &str,
        targets: &[String],
        deploy: Option<&str>,
        archive: Vec<u8>,
    ) -> Result<String> {
        let mut form = reqwest::multipart::Form::new()
            .text("tag", tag.to_string())
            .text("targets", targets.join(","))
            .part(
                "source",
                reqwest::multipart::Part::bytes(archive).file_name("source.zip"),
            );
        if let Some(d) = deploy {
            form = form.text("deploy", d.to_string());
        }
        let v = Self::ok(
            self.post("/api/admin/launcher/local/jobs")
                .multipart(form)
                .send()
                .await?,
        )
        .await?;
        v["job_id"]
            .as_str()
            .map(String::from)
            .context("the master did not return a job id")
    }

    pub async fn job_log(&self, job_id: &str) -> Result<Value> {
        let url = format!("{}/api/admin/launcher/build/{job_id}/log", self.base);
        Self::ok(self.http.get(url).bearer_auth(&self.token).send().await?).await
    }
}

pub fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}
