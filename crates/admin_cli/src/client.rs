//! HTTP-клиент к admin API мастера.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use tokio::io::AsyncWriteExt;

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

    fn req(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.http
            .request(method, self.url(path))
            .bearer_auth(&self.token)
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        Self::handle(self.req(reqwest::Method::GET, path).send().await?).await
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
        Self::handle(
            self.req(reqwest::Method::POST, path)
                .json(&body)
                .send()
                .await?,
        )
        .await
    }

    pub async fn put(&self, path: &str, body: Value) -> Result<Value> {
        Self::handle(
            self.req(reqwest::Method::PUT, path)
                .json(&body)
                .send()
                .await?,
        )
        .await
    }

    pub async fn delete(&self, path: &str) -> Result<Value> {
        Self::handle(self.req(reqwest::Method::DELETE, path).send().await?).await
    }

    /// Скачивание файла с потоковой записью на диск.
    pub async fn download(&self, path: &str, out_path: &str) -> Result<()> {
        let mut resp = self.req(reqwest::Method::GET, path).send().await?;
        let status = resp.status();
        if !status.is_success() {
            bail!("HTTP {status}: {}", resp.text().await.unwrap_or_default());
        }
        let mut file = tokio::fs::File::create(out_path)
            .await
            .with_context(|| format!("создание {out_path}"))?;
        let mut written: u64 = 0;
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
            written += chunk.len() as u64;
        }
        eprintln!("скачано {written} байт → {out_path}");
        Ok(())
    }

    /// Multipart-загрузка файла (поля `path` опционально + `file`).
    pub async fn upload(&self, path: &str, file_path: &str, extra: Option<&str>) -> Result<Value> {
        let fields = extra
            .map(|p| vec![("path".to_string(), p.to_string())])
            .unwrap_or_default();
        self.upload_fields(path, file_path, fields).await
    }

    /// Multipart-загрузка файла с произвольными текстовыми полями + `file`.
    pub async fn upload_fields(
        &self,
        path: &str,
        file: &str,
        fields: Vec<(String, String)>,
    ) -> Result<Value> {
        let part = Self::read_part(file).await?;
        let mut form = reqwest::multipart::Form::new().part("file", part);
        for (k, v) in fields {
            form = form.text(k, v);
        }
        Self::handle(
            self.req(reqwest::Method::POST, path)
                .multipart(form)
                .send()
                .await?,
        )
        .await
    }

    /// Multipart-загрузка с указанным именем поля (например, "image").
    pub async fn upload_image(&self, path: &str, file: &str, field_name: &str) -> Result<Value> {
        let part = Self::read_part(file).await?;
        let form = reqwest::multipart::Form::new().part(field_name.to_string(), part);
        Self::handle(
            self.req(reqwest::Method::POST, path)
                .multipart(form)
                .send()
                .await?,
        )
        .await
    }

    /// Чтение файла как multipart-часть.
    async fn read_part(file_path: &str) -> Result<reqwest::multipart::Part> {
        let bytes = tokio::fs::read(file_path)
            .await
            .with_context(|| format!("чтение {file_path}"))?;
        let name = std::path::Path::new(file_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "file".into());
        Ok(reqwest::multipart::Part::bytes(bytes).file_name(name))
    }

    async fn handle(resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            bail!("HTTP {status}: {}", explain(&text));
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
    }
}

/// Человеческая часть отказа мастера.
///
/// Мастер отвечает `{"error": {"code", "message"}}`. Печатать конверт целиком
/// значит показывать администратору фигурные скобки вместо причины.
fn explain(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| {
            let e = v.get("error")?;
            let message = e.get("message")?.as_str()?.to_string();
            let mut out = match e.get("code").and_then(Value::as_str) {
                Some(code) => format!("{message} [{code}]"),
                None => message,
            };
            // Отказ по форме объясняют поля: без них остаётся «request
            // validation failed», по которому нечего исправлять.
            for field in e
                .get("details")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let (Some(name), Some(text)) = (
                    field.get("field").and_then(Value::as_str),
                    field.get("message").and_then(Value::as_str),
                ) else {
                    continue;
                };
                out.push_str(&format!("\n  {name}: {text}"));
            }
            Some(out)
        })
        .unwrap_or_else(|| body.to_string())
}

/// Вывести JSON красиво.
pub fn print_json(v: &Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}
