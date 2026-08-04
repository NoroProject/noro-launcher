//! HTTP-клиент к admin API мастера.

use anyhow::{bail, Context, Result};
use serde_json::Value;

pub struct Client {
    base: String,
    token: String,
    http: reqwest::Client,
}

impl Client {
    pub fn new(base: String, token: String) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
            token,
            http: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        let resp = self
            .http
            .get(self.url(path))
            .bearer_auth(&self.token)
            .send()
            .await?;
        Self::handle(resp).await
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
        let resp = self
            .http
            .post(self.url(path))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;
        Self::handle(resp).await
    }

    pub async fn put(&self, path: &str, body: Value) -> Result<Value> {
        let resp = self
            .http
            .put(self.url(path))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;
        Self::handle(resp).await
    }

    pub async fn delete(&self, path: &str) -> Result<Value> {
        let resp = self
            .http
            .delete(self.url(path))
            .bearer_auth(&self.token)
            .send()
            .await?;
        Self::handle(resp).await
    }

    /// Multipart-загрузка файла (поля `path` опционально + `file`).
    pub async fn upload(
        &self,
        path: &str,
        file_path: &str,
        extra_path: Option<&str>,
    ) -> Result<Value> {
        let fields = extra_path
            .map(|p| vec![("path".to_string(), p.to_string())])
            .unwrap_or_default();
        self.upload_fields(path, file_path, fields).await
    }

    /// Multipart-загрузка файла с произвольными текстовыми полями + `file`.
    pub async fn upload_fields(
        &self,
        path: &str,
        file_path: &str,
        fields: Vec<(String, String)>,
    ) -> Result<Value> {
        let bytes = tokio::fs::read(file_path)
            .await
            .with_context(|| format!("чтение {file_path}"))?;
        let filename = std::path::Path::new(file_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "file".into());
        let mut form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(bytes).file_name(filename),
        );
        for (key, value) in fields {
            form = form.text(key, value);
        }
        let resp = self
            .http
            .post(self.url(path))
            .bearer_auth(&self.token)
            .multipart(form)
            .send()
            .await?;
        Self::handle(resp).await
    }

    async fn handle(resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            bail!("HTTP {status}: {text}");
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
    }
}

/// Вывести JSON красиво.
pub fn print_json(v: &Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}
