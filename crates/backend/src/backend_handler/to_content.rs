//! Window requests about the content catalogue and the player's own content.

use super::*;

impl BackendState {
    /// Only ever gets the variants `handle_to_backend` routes here.
    pub(super) async fn on_content_request(&mut self, msg: MessageToBackend) {
        match msg {
            MessageToBackend::SearchCatalog {
                query,
                provider,
                mc_version,
                loader,
                project_type,
                sort,
                offset,
            } => {
                let ctx = self.ctx.clone();
                let http = self.ctx.http.clone();
                tokio::spawn(async move {
                    let master_url = ctx.config.get().master_url;
                    let page = crate::catalog_search::search(
                        &http,
                        &master_url,
                        ctx.ws.token().as_deref(),
                        &crate::catalog_search::Query {
                            text: &query,
                            provider: &provider,
                            mc_version: mc_version.as_deref(),
                            loader: loader.as_deref(),
                            project_type: &project_type,
                            sort: &sort,
                            offset,
                        },
                    )
                    .await;
                    match page {
                        Ok(page) => ctx.send(MessageToFrontend::CatalogSearchResults {
                            hits: page.hits,
                            total: page.total,
                            offset: page.offset,
                            limit: page.limit,
                        }),
                        Err(e) => {
                            tracing::error!(error = %format!("{e:#}"), "catalog search failed");
                            ctx.send(MessageToFrontend::CatalogFailed {
                                message: format!("{e:#}"),
                            });
                        }
                    }
                });
            }
            MessageToBackend::RequestModProject {
                provider,
                project_id,
            } => {
                let ctx = self.ctx.clone();
                tokio::spawn(async move {
                    // The player's catalog endpoint: the admin one this used
                    // to call refuses anyone without staff rights.
                    let loaded = async {
                        let api = crate::master_api::MasterApi::for_session(&ctx)
                            .ok_or_else(|| anyhow::anyhow!("not signed in"))?;
                        let page = api.catalog_project(&provider, &project_id).await?;
                        // The page's fields line up with `ModProjectInfo` by
                        // name, and serde drops whatever else the master sends.
                        Ok::<_, anyhow::Error>(serde_json::from_value::<bridge::ModProjectInfo>(
                            page,
                        )?)
                    }
                    .await;
                    match loaded {
                        Ok(project) => ctx.send(MessageToFrontend::ModProjectLoaded { project }),
                        Err(e) => {
                            tracing::error!(error = %format!("{e:#}"), "mod page failed to load");
                            ctx.send(MessageToFrontend::CatalogFailed {
                                message: format!("{e:#}"),
                            });
                        }
                    }
                });
            }
            MessageToBackend::RequestPersonalContent { server_id } => {
                crate::personal::request(&self.ctx, server_id)
            }
            MessageToBackend::RequestContentVersions {
                provider,
                project_id,
                server_id,
            } => {
                // The build decides which versions fit, so its Minecraft
                // version and loader travel with the request rather than being
                // guessed from the project.
                let (mc_version, loader) = self
                    .servers
                    .iter()
                    .find(|s| s.id == server_id)
                    .map(|s| (s.mc_version.clone(), s.modloader.as_str().to_string()))
                    .unwrap_or_default();
                crate::personal::request_versions(
                    &self.ctx, provider, project_id, mc_version, loader,
                )
            }
            MessageToBackend::InstallPersonalContent {
                server_id,
                kind,
                provider,
                project_id,
                version_id,
                title,
                icon_url,
            } => crate::personal::install(
                &self.ctx, server_id, kind, provider, project_id, version_id, title, icon_url,
            ),
            MessageToBackend::RemovePersonalContent { server_id, id } => {
                crate::personal::remove(&self.ctx, server_id, id)
            }
            MessageToBackend::SetPersonalContentEnabled {
                server_id,
                id,
                enabled,
            } => crate::personal::set_enabled(&self.ctx, server_id, id, enabled),
            _ => {}
        }
    }
}
