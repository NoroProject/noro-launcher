//! Настройка экземпляра `Webauthn` из конфигурации мастера.
//!
//! Домен (`rp_id`) и список разрешённых origin'ов — это то, что проверяется при
//! входе. Ошибка здесь означает либо неработающий вход, либо принятие подписи,
//! сделанной на чужом сайте.

use crate::config::Config;
use anyhow::{Context, Result};
use url::Url;
use webauthn_rs::prelude::*;

/// Собрать `Webauthn` для мастера.
///
/// `rp_id` берётся из адреса сайта, а не API: passkey привязывается к домену
/// навсегда и не отзывается — ошибка тут не лечится, ключи у игроков просто
/// перестают работать. Оба origin'а (сайт и API) добавляются явно: страница
/// входа лаунчера отдаётся с API-домена.
pub fn build(config: &Config) -> Result<Webauthn> {
    let web_origin = Url::parse(&config.web_url)
        .with_context(|| format!("NORO_WEB_URL ({}) не разобрать как URL", config.web_url))?;
    let api_origin = Url::parse(&config.public_url).with_context(|| {
        format!(
            "NORO_PUBLIC_URL ({}) не разобрать как URL",
            config.public_url
        )
    })?;

    let rp_id = rp_id_for(&web_origin, &api_origin)?;

    let mut builder = WebauthnBuilder::new(&rp_id, &web_origin)
        .context("не собрать WebAuthn: домен и адрес сайта не сходятся")?
        .rp_name("Noro Network");
    if api_origin != web_origin {
        builder = builder.append_allowed_origin(&api_origin);
    }
    // API живёт на поддомене сайта (`api.example.dev` при rp_id `example.dev`),
    // без этого его origin отвергается.
    builder = builder.allow_subdomains(true);

    builder.build().context("не собрать WebAuthn")
}

/// Домен, к которому браузер привяжет ключ: общий хвост адресов сайта и API.
///
/// `api.example.dev` + `example.dev` → `example.dev`. Если общего хвоста нет
/// (разные домены), passkey работать не может, и об этом лучше узнать при
/// старте, а не при первом входе игрока.
fn rp_id_for(web: &Url, api: &Url) -> Result<String> {
    let web_host = web
        .host_str()
        .with_context(|| format!("в NORO_WEB_URL ({web}) нет домена"))?
        .to_string();
    let api_host = api
        .host_str()
        .with_context(|| format!("в NORO_PUBLIC_URL ({api}) нет домена"))?;

    if api_host == web_host || api_host.ends_with(&format!(".{web_host}")) {
        return Ok(web_host);
    }
    // Сайт на поддомене API-домена — редкий, но валидный случай.
    if web_host.ends_with(&format!(".{api_host}")) {
        return Ok(api_host.to_string());
    }

    anyhow::bail!(
        "у сайта ({web_host}) и API ({api_host}) нет общего домена — passkey \
         работать не будет; поднимите их на одном домене"
    )
}

#[cfg(test)]
#[path = "webauthn_tests.rs"]
mod tests;
