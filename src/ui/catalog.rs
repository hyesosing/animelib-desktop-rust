// src/ui/catalog.rs
use crate::api::{models::AnimeItem, ApiClient};
use crate::app::{NavAction, Screen};
use crate::cache::ImageCache;
use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};
use tokio::runtime::Runtime;

/// Color for rating badge text based on score value
fn rating_color(rating_str: &str) -> egui::Color32 {
    if let Ok(val) = rating_str.parse::<f32>() {
        if val >= 8.0 {
            egui::Color32::from_rgb(76, 175, 80)   // Green
        } else if val >= 6.0 {
            egui::Color32::from_rgb(255, 183, 77)  // Amber
        } else {
            egui::Color32::from_rgb(239, 83, 80)   // Red
        }
    } else {
        egui::Color32::from_rgb(158, 158, 158)
    }
}

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
        // ── Poll async results ───────────────────────────────────
        if let Some(rx) = &self.rx {
            if let Ok(res) = rx.try_recv() {
                self.is_loading = false;
                self.rx = None;
                match res {
                    Ok(resp) => {
                        self.items = resp.data;
                        self.current_page = resp.meta.current_page;
                        if let Some(lp) = resp.meta.last_page {
                            self.total_pages = lp;
                        } else if self.items.len() == 60 {
                            self.total_pages = self.current_page + 1;
                        } else {
                            self.total_pages = self.current_page;
                        }
                    }
                    Err(e) => {
                        self.error_msg = Some(format!("Ошибка загрузки: {}", e));
                    }
                }
            }
        }

        // Auto-load first page
        if self.items.is_empty() && !self.is_loading && self.rx.is_none() && self.error_msg.is_none() {
            self.load_page(client.clone(), rt.clone(), ui.ctx(), self.current_page);
        }

        // ── Error state ──────────────────────────────────────────
        if let Some(err) = self.error_msg.clone() {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.label(
                        egui::RichText::new(&err)
                            .color(egui::Color32::from_rgb(239, 83, 80))
                            .size(16.0),
                    );
                    ui.add_space(20.0);
                    let retry = egui::Button::new(
                        egui::RichText::new("Повторить").size(14.0).color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(126, 87, 194))
                    .rounding(8.0)
                    .min_size(egui::vec2(160.0, 42.0));
                    if ui.add(retry).clicked() {
                        self.error_msg = None;
                    }
                });
            });
            return;
        }

        // ── Loading state ────────────────────────────────────────
        if self.is_loading {
            ui.centered_and_justified(|ui| {
                ui.add(egui::Spinner::new().size(48.0));
            });
            return;
        }

        // ── Main catalog grid ────────────────────────────────────
        let bg_color = ui.visuals().panel_fill;

        egui::ScrollArea::vertical().show(ui, |ui| {
            let h_pad = 32.0_f32;
            let available_w = ui.available_width();
            let content_w = (available_w - h_pad * 2.0).max(200.0);

            let min_card = 155.0_f32;
            let gap = 18.0_f32;
            let cols = ((content_w + gap) / (min_card + gap)).floor().max(1.0) as usize;
            let total_gaps = gap * (cols as f32 - 1.0);
            let card_w = ((content_w - total_gaps) / cols as f32).floor().max(min_card);
            let poster_h = (card_w * 1.42).floor();

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.add_space(h_pad);

                egui::Grid::new("catalog_grid")
                    .num_columns(cols)
                    .spacing([gap, 28.0])
                    .show(ui, |ui| {
                        for (i, item) in self.items.iter().enumerate() {
                            let img_size = egui::vec2(card_w, poster_h);

                            let card_resp = ui.vertical(|ui| {
                                ui.set_width(card_w);

                                // ── Poster image ─────────────────
                                let image_url = item
                                    .cover
                                    .as_ref()
                                    .and_then(|c| c.default.clone())
                                    .unwrap_or_default();

                                let poster_min = ui.cursor().min;

                                {
                                    let mut cache = image_cache.lock().unwrap();
                                    ImageCache::render_image(
                                        ui,
                                        &mut cache,
                                        &image_url,
                                        client.as_ref(),
                                        rt.clone(),
                                        img_size,
                                    );
                                }

                                let poster_rect =
                                    egui::Rect::from_min_size(poster_min, img_size);
                                let painter = ui.painter();

                                // ── Rounded corner mask ──────────
                                // Paint a thick border matching background to hide sharp image corners
                                painter.rect_stroke(
                                    poster_rect.expand(1.5),
                                    egui::Rounding::same(10.0),
                                    egui::Stroke::new(6.0, bg_color),
                                );

                                // ── Rating badge overlay ─────────
                                if let Some(ri) = &item.rating {
                                    if let Some(avg) = &ri.average {
                                        if !avg.is_empty() {
                                            let badge_w = 42.0_f32;
                                            let badge_h = 24.0_f32;
                                            let badge_pos =
                                                poster_rect.min + egui::vec2(8.0, 8.0);
                                            let badge_rect = egui::Rect::from_min_size(
                                                badge_pos,
                                                egui::vec2(badge_w, badge_h),
                                            );

                                            // Dark pill background
                                            painter.rect_filled(
                                                badge_rect,
                                                egui::Rounding::same(6.0),
                                                egui::Color32::from_rgba_unmultiplied(
                                                    0, 0, 0, 190,
                                                ),
                                            );

                                            // Colored rating number
                                            painter.text(
                                                badge_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                avg,
                                                egui::FontId::new(
                                                    12.0,
                                                    egui::FontFamily::Proportional,
                                                ),
                                                rating_color(avg),
                                            );
                                        }
                                    }
                                }

                                ui.add_space(10.0);

                                // ── Title ────────────────────────
                                let title =
                                    item.rus_name.as_deref().unwrap_or(&item.name);
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(title)
                                            .strong()
                                            .size(13.5)
                                            .color(egui::Color32::from_rgb(
                                                235, 235, 240,
                                            )),
                                    )
                                    .truncate(true),
                                );

                                // ── Type label ───────────────────
                                if let Some(t) = &item.item_type {
                                    ui.label(
                                        egui::RichText::new(&t.label)
                                            .size(12.0)
                                            .color(egui::Color32::from_rgb(
                                                120, 120, 128,
                                            )),
                                    );
                                }
                            })
                            .response;

                            // ── Card interaction ─────────────────
                            let interact = ui.interact(
                                card_resp.rect,
                                ui.id().with(("card", item.id)),
                                egui::Sense::click(),
                            );

                            if interact.clicked() {
                                *nav = NavAction::GoTo(Screen::AnimePage(
                                    item.slug.clone(),
                                ));
                            }

                            if interact.hovered() {
                                ui.ctx().set_cursor_icon(
                                    egui::CursorIcon::PointingHand,
                                );
                            }

                            // ── Hover popover ────────────────────
                            interact.on_hover_ui_at_pointer(|ui| {
                                Self::render_popover(ui, item);
                            });

                            if (i + 1) % cols == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });

            // ── Pagination ───────────────────────────────────────
            ui.add_space(36.0);
            ui.horizontal(|ui| {
                ui.add_space(h_pad);

                let btn_size = egui::vec2(120.0, 40.0);

                if self.current_page > 1 {
                    let prev = egui::Button::new(
                        egui::RichText::new("← Назад")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(200, 200, 205)),
                    )
                    .rounding(10.0)
                    .min_size(btn_size);
                    if ui.add(prev).clicked() {
                        self.load_page(
                            client.clone(),
                            rt.clone(),
                            ui.ctx(),
                            self.current_page - 1,
                        );
                    }
                }

                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} / {}",
                        self.current_page, self.total_pages
                    ))
                    .size(15.0)
                    .color(egui::Color32::from_rgb(120, 120, 128)),
                );
                ui.add_space(12.0);

                if self.current_page < self.total_pages {
                    let next = egui::Button::new(
                        egui::RichText::new("Далее →")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(200, 200, 205)),
                    )
                    .rounding(10.0)
                    .min_size(btn_size);
                    if ui.add(next).clicked() {
                        self.load_page(
                            client.clone(),
                            rt.clone(),
                            ui.ctx(),
                            self.current_page + 1,
                        );
                    }
                }
            });
            ui.add_space(32.0);
        });
    }

    /// Rich hover popover matching AnimeLib's tooltip design
    fn render_popover(ui: &mut egui::Ui, item: &AnimeItem) {
        ui.set_min_width(340.0);
        ui.set_max_width(340.0);

        // ── Title ────────────────────────────────────────────────
        let title = item.rus_name.as_deref().unwrap_or(&item.name);
        ui.label(
            egui::RichText::new(title)
                .strong()
                .size(17.0)
                .color(egui::Color32::WHITE),
        );

        // ── Original name ────────────────────────────────────────
        ui.label(
            egui::RichText::new(&item.name)
                .size(13.0)
                .color(egui::Color32::from_rgb(120, 120, 128)),
        );

        ui.add_space(14.0);

        // ── Info table ───────────────────────────────────────────
        egui::Frame::none()
            .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 6))
            .rounding(8.0)
            .inner_margin(egui::Margin::symmetric(14.0, 10.0))
            .show(ui, |ui| {
                egui::Grid::new(ui.id().with("pop_info"))
                    .num_columns(3)
                    .spacing([28.0, 4.0])
                    .show(ui, |ui| {
                        for h in ["Статус", "Выпуск", "Эпизоды"] {
                            ui.label(
                                egui::RichText::new(h)
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 100, 108)),
                            );
                        }
                        ui.end_row();

                        let status = item
                            .status
                            .as_ref()
                            .map(|s| s.label.as_str())
                            .unwrap_or("—");
                        let eps = item
                            .episodes_count
                            .map(|e| format!("{} эп.", e))
                            .unwrap_or_else(|| "—".into());

                        let vals: [&str; 3] = [status, "—", &eps];
                        for v in vals {
                            ui.label(
                                egui::RichText::new(v)
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(220, 220, 228)),
                            );
                        }
                    });
            });

        ui.add_space(14.0);

        // ── Tags ─────────────────────────────────────────────────
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

            // Age restriction
            if let Some(age) = &item.age_restriction {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(239, 83, 80),
                    ))
                    .rounding(4.0)
                    .inner_margin(egui::vec2(8.0, 3.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&age.label)
                                .size(11.0)
                                .color(egui::Color32::from_rgb(239, 83, 80)),
                        );
                    });
            }

            // Type tag
            if let Some(t) = &item.item_type {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10))
                    .rounding(4.0)
                    .inner_margin(egui::vec2(8.0, 3.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&t.label)
                                .size(11.0)
                                .color(egui::Color32::from_rgb(190, 190, 198)),
                        );
                    });
            }

            // Rating tag
            if let Some(ri) = &item.rating {
                if let Some(avg) = &ri.average {
                    if !avg.is_empty() {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10))
                            .rounding(4.0)
                            .inner_margin(egui::vec2(8.0, 3.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!("★ {}", avg))
                                        .size(11.0)
                                        .color(rating_color(avg)),
                                );
                            });
                    }
                }
            }
        });

        ui.add_space(14.0);

        // ── Action button ────────────────────────────────────────
        let btn = egui::Button::new(
            egui::RichText::new("В планы")
                .size(14.0)
                .color(egui::Color32::WHITE),
        )
        .fill(egui::Color32::from_rgb(126, 87, 194))
        .rounding(8.0)
        .min_size(egui::vec2(ui.available_width(), 38.0));
        ui.add(btn);
    }

    fn load_page(
        &mut self,
        client: Arc<ApiClient>,
        rt: Arc<Runtime>,
        ctx: &egui::Context,
        page: u32,
    ) {
        self.is_loading = true;
        let query = if self.search_query.is_empty() {
            None
        } else {
            Some(self.search_query.clone())
        };
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
