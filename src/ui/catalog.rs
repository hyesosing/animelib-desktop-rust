// src/ui/catalog.rs
use crate::api::{models::AnimeItem, ApiClient};
use crate::app::{NavAction, Screen};
use crate::cache::ImageCache;
use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};
use tokio::runtime::Runtime;

pub struct CatalogScreen {
    items: Vec<AnimeItem>,
    is_loading: bool,
    current_page: u32,
    total_pages: u32,
    search_query: String,
    rx: Option<mpsc::Receiver<anyhow::Result<crate::api::models::AnimeCatalogResponse>>>,
    error_msg: Option<String>,
}

impl CatalogScreen {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            is_loading: false,
            current_page: 1,
            total_pages: 1,
            search_query: String::new(),
            rx: None,
            error_msg: None,
        }
    }

    pub fn set_search(&mut self, query: String) {
        self.search_query = query;
        self.current_page = 1;
        self.items.clear();
        self.is_loading = false;
        self.error_msg = None;
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        client: Arc<ApiClient>,
        image_cache: Arc<Mutex<ImageCache>>,
        nav: &mut NavAction,
        rt: Arc<Runtime>,
    ) {
        if let Some(rx) = &self.rx {
            if let Ok(res) = rx.try_recv() {
                self.is_loading = false;
                self.rx = None;
                match res {
                    Ok(resp) => {
                        self.items = resp.data;
                        self.current_page = resp.meta.current_page;
                        
                        // If last_page is not provided, estimate if we can go to next page
                        if let Some(lp) = resp.meta.last_page {
                            self.total_pages = lp;
                        } else if self.items.len() == 60 { // assuming per_page is 60
                            self.total_pages = self.current_page + 1;
                        } else {
                            self.total_pages = self.current_page;
                        }
                    }
                    Err(e) => {
                        let err = format!("Failed to load catalog: {}", e);
                        log::error!("{}", err);
                        self.error_msg = Some(err);
                    }
                }
            }
        }

        if self.items.is_empty() && !self.is_loading && self.rx.is_none() && self.error_msg.is_none() {
            self.load_page(client.clone(), rt.clone(), ui.ctx(), self.current_page);
        }

        if let Some(err) = self.error_msg.clone() {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                    ui.add_space(10.0);
                    if ui.button("Повторить попытку").clicked() {
                        self.error_msg = None;
                    }
                });
            });
            return;
        }

        if self.is_loading {
            ui.centered_and_justified(|ui| {
                ui.add(egui::Spinner::new().size(40.0));
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);
            
            let panel_width = ui.available_width();
            let columns = (panel_width / 180.0).max(1.0) as usize;
            
            egui::Grid::new("catalog_grid")
                .num_columns(columns)
                .spacing([20.0, 20.0])
                .show(ui, |ui| {
                    for (i, item) in self.items.iter().enumerate() {
                        ui.vertical(|ui| {
                            let image_url = item.cover.as_ref().and_then(|c| c.default.clone()).unwrap_or_default();
                            
                            let mut cache = image_cache.lock().unwrap();
                            ImageCache::render_image(ui, &mut cache, &image_url, client.as_ref(), rt.clone(), egui::vec2(160.0, 220.0));
                            drop(cache);
                            
                            ui.add_space(4.0);
                            let title = item.rus_name.as_deref().unwrap_or(&item.name);
                            let title_label = egui::Label::new(
                                egui::RichText::new(title).strong().size(14.0)
                            ).truncate(true);
                            
                            let resp = ui.add(title_label);
                            let rect = ui.min_rect();
                            let interact_resp = ui.interact(rect, ui.id().with(item.id), egui::Sense::click());
                            
                            if resp.clicked() || interact_resp.clicked() {
                                *nav = NavAction::GoTo(Screen::AnimePage(item.slug.clone()));
                            }
                            
                            ui.horizontal(|ui| {
                                if let Some(t) = &item.item_type {
                                    ui.label(egui::RichText::new(&t.label).color(egui::Color32::LIGHT_BLUE));
                                }
                                if let Some(ep) = item.episodes_count {
                                    ui.label(format!("{} эп.", ep));
                                }
                                if let Some(rating_info) = &item.rating {
                                    if let Some(avg) = &rating_info.average {
                                        ui.label(format!("⭐ {}", avg));
                                    }
                                }
                            });
                        });
                        
                        if (i + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
                
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if self.current_page > 1 {
                    if ui.button("⬅ Предыдущая").clicked() {
                        self.load_page(client.clone(), rt.clone(), ui.ctx(), self.current_page - 1);
                    }
                }
                ui.label(format!("Страница {} из {}", self.current_page, self.total_pages));
                if self.current_page < self.total_pages {
                    if ui.button("Следующая ➡").clicked() {
                        self.load_page(client.clone(), rt.clone(), ui.ctx(), self.current_page + 1);
                    }
                }
            });
            ui.add_space(20.0);
        });
    }

    fn load_page(&mut self, client: Arc<ApiClient>, rt: Arc<Runtime>, ctx: &egui::Context, page: u32) {
        self.is_loading = true;
        let query = if self.search_query.is_empty() { None } else { Some(self.search_query.clone()) };
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        let ctx_clone = ctx.clone();
        
        rt.spawn(async move {
            let res = client.get_catalog(page, query.as_deref()).await;
            let _ = tx.send(res);
            ctx_clone.request_repaint();
        });
    }
}
