// src/cache/image_cache.rs
use crate::api::ApiClient;
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub enum CachedImage {
    Loading,
    Loaded(egui::TextureHandle),
    Failed,
}

pub struct ImageCache {
    cache: HashMap<String, Arc<Mutex<CachedImage>>>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get_or_load(
        &mut self,
        url: &str,
        client: &ApiClient,
        ctx: &egui::Context,
        rt: Arc<Runtime>,
    ) -> Arc<Mutex<CachedImage>> {
        if let Some(cached) = self.cache.get(url) {
            return cached.clone();
        }

        let cached_image = Arc::new(Mutex::new(CachedImage::Loading));
        self.cache.insert(url.to_string(), cached_image.clone());

        let url_clone = url.to_string();
        let client_clone = client.clone();
        let ctx_clone = ctx.clone();
        let cached_image_clone = cached_image.clone();

        rt.spawn(async move {
            match client_clone.fetch_image_bytes(&url_clone).await {
                Ok(bytes) => {
                    if let Ok(image) = image::load_from_memory(&bytes) {
                        let size = [image.width() as _, image.height() as _];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            pixels.as_slice(),
                        );

                        let texture = ctx_clone.load_texture(
                            &url_clone,
                            color_image,
                            egui::TextureOptions::default(),
                        );

                        *cached_image_clone.lock().unwrap() = CachedImage::Loaded(texture);
                        ctx_clone.request_repaint();
                    } else {
                        *cached_image_clone.lock().unwrap() = CachedImage::Failed;
                        ctx_clone.request_repaint();
                    }
                }
                Err(_) => {
                    *cached_image_clone.lock().unwrap() = CachedImage::Failed;
                    ctx_clone.request_repaint();
                }
            }
        });

        cached_image
    }

    pub fn render_image(
        ui: &mut egui::Ui,
        cache: &mut ImageCache,
        url: &str,
        client: &ApiClient,
        rt: Arc<Runtime>,
        size: egui::Vec2,
    ) {
        if url.is_empty() {
            let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
            ui.painter().rect_filled(rect, 4.0, egui::Color32::DARK_GRAY);
            return;
        }

        let cached_arc = cache.get_or_load(url, client, ui.ctx(), rt);
        let guard = cached_arc.lock().unwrap();

        match &*guard {
            CachedImage::Loaded(texture) => {
                ui.add(egui::Image::new(texture).fit_to_exact_size(size));
            }
            CachedImage::Loading => {
                let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, egui::Color32::from_gray(40));
                
                let spinner_size = 20.0;
                let center = rect.center();
                ui.put(
                    egui::Rect::from_center_size(center, egui::vec2(spinner_size, spinner_size)),
                    egui::Spinner::new().size(spinner_size)
                );
            }
            CachedImage::Failed => {
                let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, egui::Color32::DARK_RED);
            }
        }
    }
}
