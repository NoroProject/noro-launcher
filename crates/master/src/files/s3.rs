//! Загрузка файлов в S3-совместимые хранилища (AWS S3, Cloudflare R2, MinIO).
//! Использует AWS Signature V4 через уже имеющиеся crates hmac + sha2.
use crate::config::S3Config;
use anyhow::{Context, Result};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// Загрузить файл в S3 и вернуть публичный URL.
pub async fn put(
    client: &reqwest::Client,
    cfg: &S3Config,
    sha1: &str,
    data: &[u8],
) -> Result<String> {
    let now = Utc::now();
    let date = now.format("%Y%m%d").to_string();
    let datetime = now.format("%Y%m%dT%H%M%SZ").to_string();

    let host = endpoint_host(&cfg.endpoint);
    let key = sha1;
    let path = format!("/{key}");
    let content_sha256 = hex::encode(Sha256::digest(data));
    let content_type = "application/octet-stream";

    // Canonical headers (sorted)
    let canonical_headers = format!(
        "content-type:{content_type}\nhost:{host}\nx-amz-content-sha256:{content_sha256}\nx-amz-date:{datetime}\n"
    );
    let signed_headers = "content-type;host;x-amz-content-sha256;x-amz-date";

    let canonical_request =
        format!("PUT\n{path}\n\n{canonical_headers}\n{signed_headers}\n{content_sha256}");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{datetime}\n{date}/{}/{}/aws4_request\n{}",
        cfg.region,
        "s3",
        hex::encode(Sha256::digest(canonical_request.as_bytes()))
    );

    let signing_key = derive_signing_key(&cfg.secret_key, &date, &cfg.region);
    let mut mac = HmacSha256::new_from_slice(&signing_key).unwrap();
    mac.update(string_to_sign.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}/{}/s3/aws4_request,SignedHeaders={signed_headers},Signature={signature}",
        cfg.access_key, date, cfg.region
    );

    let url = format!("{}/{}", cfg.endpoint.trim_end_matches('/'), key);
    client
        .put(&url)
        .header("host", &host)
        .header("content-type", content_type)
        .header("x-amz-content-sha256", &content_sha256)
        .header("x-amz-date", &datetime)
        .header("authorization", &authorization)
        .body(data.to_vec())
        .send()
        .await
        .context("S3 PUT request")?
        .error_for_status()
        .context("S3 PUT status")?;

    Ok(format!("{}/{}", cfg.public_url, key))
}

fn endpoint_host(endpoint: &str) -> String {
    endpoint
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}

fn derive_signing_key(secret: &str, date: &str, region: &str) -> Vec<u8> {
    let key = format!("AWS4{secret}");
    let k_date = hmac_sha256(key.as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    hmac_sha256(&k_service, b"aws4_request")
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}
