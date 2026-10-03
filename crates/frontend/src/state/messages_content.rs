// Over 150 lines: one handler for every content message; split, the match would
// only move.
//! Backend messages about news, the content catalogue, skins and capes.

use super::*;

impl LauncherUI {
    /// Only ever gets the variants `on_message` routes here.
    pub(super) fn on_content_message(&mut self, msg: MessageToFrontend, cx: &mut Context<Self>) {
        match msg {
            MessageToFrontend::NewsUpdated { items } => {
                let ids: HashSet<Uuid> = items.iter().map(|n| n.id).collect();
                self.news_images.retain(|id, _| ids.contains(id));
                self.news_excerpts = items
                    .iter()
                    .map(|n| (n.id, crate::pages::plain_excerpt(&n.body, 240).into()))
                    .collect();
                self.news = items;
                self.news_loaded = true;
            }
            MessageToFrontend::CatalogSearchResults {
                hits,
                total,
                offset,
                limit,
            } => {
                self.mod_catalog_hits = hits;
                self.mod_catalog_total = total;
                self.mod_catalog_offset = offset;
                self.mod_catalog_limit = limit;
                self.mod_catalog_error = None;
                self.content_searching = false;
            }
            MessageToFrontend::CatalogFailed { message } => {
                self.content_searching = false;
                self.mod_catalog_error = Some(message);
            }
            MessageToFrontend::ModProjectLoaded { project } => {
                // The reply can land after the player has moved to another mod.
                let still_open = self
                    .mod_catalog_selected
                    .as_ref()
                    .is_some_and(|s| s.project_id == project.project_id);
                if still_open {
                    self.mod_project = Some(project);
                }
            }
            MessageToFrontend::PersonalContent { server_id, items } => {
                self.personal_content.insert(server_id, items);
                self.content_busy = false;
                self.content_error = None;
            }
            MessageToFrontend::ContentVersions {
                provider,
                project_id,
                versions,
            } => {
                self.content_versions
                    .insert((provider, project_id), versions);
                self.content_busy = false;
            }
            MessageToFrontend::ContentActionFailed { message } => {
                self.content_busy = false;
                self.java_busy = false;
                self.content_error = Some(message);
            }

            MessageToFrontend::SkinUploadFailed => {
                self.skin_uploading = false;
                self.skin_bytes = self.skin_before_upload.take();
            }
            MessageToFrontend::CapesList { capes } => {
                self.capes = capes.clone();
                let master_url = self.config.master_url.clone();
                // A cape's render doesn't change; every reconnect used to
                // download all of them again.
                let have: HashSet<Uuid> = self.cape_images.keys().copied().collect();
                for cape in capes.into_iter().filter(|c| !have.contains(&c.id)) {
                    let id = cape.id;
                    // The texture address goes inside a query string, so it
                    // has to be escaped like one.
                    let render_url = format!(
                        "{}/api/textures/renders/cape?url={}&scale=10",
                        master_url.trim_end_matches('/'),
                        urlencoding::encode(&cape.url)
                    );
                    cx.spawn(async move |this, cx| {
                        if let Ok(img) =
                            crate::image_loader::load_image_capped(render_url, PRESET_RENDER_SIDE)
                                .await
                        {
                            let _ = this.update(cx, |this, cx| {
                                this.cape_images.insert(id, img);
                                cx.notify();
                            });
                        }
                    })
                    .detach();
                }
            }
            MessageToFrontend::SkinPresetsList { presets } => {
                // A preset that was already here keeps its skin and render;
                // only new ones are downloaded. Every list used to refetch all.
                let mut previous: HashMap<String, Arc<Vec<u8>>> = self
                    .custom_presets
                    .drain(..)
                    .map(|p| (p.id, p.bytes))
                    .collect();
                let master_url = self.config.master_url.clone();
                for p in presets {
                    let id = p.id;
                    let name = p.name;
                    let url = p.skin_url;
                    let known = previous.remove(&id).filter(|bytes| !bytes.is_empty());
                    let had_render = self.preset_images.contains_key(&id);
                    self.custom_presets.push(SavedSkinPreset {
                        id: id.clone(),
                        name: name.clone(),
                        bytes: known.clone().unwrap_or_default(),
                    });
                    if known.is_some() && had_render {
                        continue;
                    }

                    let url_bytes = url.clone();
                    let id_bytes = id.clone();
                    cx.spawn(async move |this, cx| {
                        if let Ok((_, bytes)) =
                            crate::image_loader::load_image_and_bytes(url_bytes).await
                        {
                            let _ = this.update(cx, |this, cx| {
                                if let Some(found) =
                                    this.custom_presets.iter_mut().find(|cp| cp.id == id_bytes)
                                {
                                    found.bytes = Arc::new(bytes);
                                }
                                cx.notify();
                            });
                        }
                    })
                    .detach();

                    let render_url = format!(
                        "{}/api/textures/renders/bust?url={}&scale=8&yaw=-25&pitch=12",
                        master_url.trim_end_matches('/'),
                        urlencoding::encode(&url)
                    );
                    let id_render = id.clone();
                    cx.spawn(async move |this, cx| {
                        if let Ok(img) =
                            crate::image_loader::load_image_capped(render_url, PRESET_RENDER_SIDE)
                                .await
                        {
                            let _ = this.update(cx, |this, cx| {
                                this.preset_images.insert(id_render, img);
                                cx.notify();
                            });
                        }
                    })
                    .detach();
                }
                // Presets deleted elsewhere: their renders go with them.
                for id in previous.into_keys() {
                    self.preset_images.remove(&id);
                }
            }
            _ => {}
        }
    }
}
