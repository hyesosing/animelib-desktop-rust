// src/ui/anime_page.rs
use crate::api::{models::{AnimeDetail, EpisodeItem}, ApiClient};
use crate::app::NavAction;
use crate::cache::ImageCache;
use super::player_launcher;
use eframe::egui;
use std::sync::{mpsc, Arc, Mutex};
use tokio::runtime::Runtime;

pub struct AnimePageScreen {
    slug: String,
    detail: Option<AnimeDetail>,
    episodes: Vec<EpisodeItem>,
    #[allow(dead_code)]
    is_loading: bool,
    selected_episode: Option<usize>,
    selected_player: Option<usize>,
    rx_detail: Option<mpsc::Receiver<anyhow::Result<AnimeDetail>>>,
    error_msg: Option<String>,
    rx_players: Option<mpsc::Receiver<(usize, anyhow::Result<Vec<crate::api::models::Player>>)>>,
    loading_players_for: Option<usize>,
    player_process: Arc<Mutex<Option<std::process::Child>>>,
}

fn extract_summary(val: &serde_json::Value) -> String {
    if let Some(content) = val.get("content").and_then(|c| c.as_array()) {
        let mut res = String::new();
        for block in content {
            if let Some(inner) = block.get("content").and_then(|c| c.as_array()) {
                for text_node in inner {
                    if let Some(s) = text_node.get("text").and_then(|t| t.as_str()) {
                        res.push_str(s);
                        res.push('\n');
                    }
                }
            }
        }
        let trimmed = res.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    // fallback if structure differs
    if let Some(s) = val.as_str() {
        return s.to_string();
    }
    String::new()
}

impl AnimePageScreen {
    pub fn new(slug: String) -> Self {
        Self {
            slug,
            detail: None,
            episodes: Vec::new(),
            is_loading: true,
            selected_episode: None,
            selected_player: None,
            rx_detail: None,
            error_msg: None,
            rx_players: None,
            loading_players_for: None,
            player_process: Arc::new(Mutex::new(None)),
        }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        client: Arc<ApiClient>,
        image_cache: Arc<Mutex<ImageCache>>,
        _nav: &mut NavAction,
        rt: Arc<Runtime>,
        hwnd: Option<isize>,
    ) {
        let mut is_playing = false;
        if let Ok(mut lock) = self.player_process.lock() {
            if let Some(mut child) = lock.take() {
                if let Ok(Some(_status)) = child.try_wait() {
                    // process ended
                } else {
                    // process still running
                    is_playing = true;
                    *lock = Some(child);
                }
            }
        }
        
        if is_playing {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Плеер запущен");
                    ui.add_space(20.0);
                    if ui.button("Остановить видео").clicked() {
                        if let Ok(mut lock) = self.player_process.lock() {
                            if let Some(mut child) = lock.take() {
                                let _ = child.kill();
                                let _ = child.wait();
                            }
                        }
                    }
                });
            });
            return;
        }

        if self.rx_detail.is_none() && self.detail.is_none() && self.error_msg.is_none() {
            let (tx1, rx1) = mpsc::channel();
            self.rx_detail = Some(rx1);
            let slug = self.slug.clone();
            let client1 = client.clone();
            let ctx = ui.ctx().clone();
            
            rt.spawn(async move {
                let res = client1.get_anime(&slug).await;
                let _ = tx1.send(res);
                ctx.request_repaint();
            });
        }

        if let Some(rx) = &self.rx_detail {
            if let Ok(res) = rx.try_recv() {
                self.rx_detail = None;
                match res {
                    Ok(d) => self.detail = Some(d),
                    Err(e) => {
                        let err = format!("Failed to load anime detail: {}", e);
                        log::error!("{}", err);
                        self.error_msg = Some(err);
                    }
                }
            }
        }

        if let Some(rx) = &self.rx_players {
            if let Ok((idx, res)) = rx.try_recv() {
                self.rx_players = None;
                self.loading_players_for = None;
                if let Ok(players) = res {
                    if let Some(detail) = &mut self.detail {
                        if let Some(eps) = &mut detail.episodes {
                            if let Some(ep) = eps.get_mut(idx) {
                                ep.players = players;
                            }
                        }
                    }
                }
            }
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

        if self.rx_detail.is_some() {
            ui.centered_and_justified(|ui| {
                ui.add(egui::Spinner::new().size(40.0));
            });
            return;
        }

        if let Some(detail) = &self.detail {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        let image_url = detail.cover.as_ref().and_then(|c| c.default.clone()).unwrap_or_default();
                        let mut cache = image_cache.lock().unwrap();
                        ImageCache::render_image(ui, &mut cache, &image_url, client.as_ref(), rt.clone(), egui::vec2(240.0, 340.0));
                        drop(cache);
                        
                        ui.add_space(10.0);
                        if let Some(status) = &detail.status {
                            ui.label(format!("Статус: {}", status.label));
                        }
                        if let Some(rating_info) = &detail.rating {
                            if let Some(avg) = &rating_info.average {
                                ui.label(format!("Рейтинг: ⭐ {}", avg));
                            }
                        }
                        if let Some(genres) = &detail.genres {
                            let g: Vec<String> = genres.iter().map(|g| g.name.clone()).collect();
                            ui.label(format!("Жанры: {}", g.join(", ")));
                        }
                    });

                    ui.add_space(20.0);

                    ui.vertical(|ui| {
                        let title = detail.rus_name.as_deref().unwrap_or(&detail.name);
                        ui.heading(title);
                        if let Some(eng) = detail.rus_name.as_ref().map(|_| &detail.name) {
                            ui.label(egui::RichText::new(eng).color(egui::Color32::GRAY));
                        }

                        ui.add_space(10.0);
                        if let Some(summary) = &detail.summary {
                            let text = extract_summary(summary);
                            if !text.is_empty() {
                                ui.label(text);
                            }
                        }

                        ui.add_space(20.0);
                        ui.heading("Эпизоды");
                        
                        let episodes = detail.episodes.as_ref().unwrap_or(&self.episodes);
                        for (i, ep) in episodes.iter().enumerate() {
                            let ep_name = ep.name.as_deref().unwrap_or("");
                            let btn_text = format!("Эпизод {} {}", ep.number, ep_name);
                            if ui.selectable_label(self.selected_episode == Some(i), btn_text).clicked() {
                                if self.selected_episode == Some(i) {
                                    self.selected_episode = None;
                                } else {
                                    self.selected_episode = Some(i);
                                    self.selected_player = None;
                                    
                                    if ep.players.is_empty() {
                                        self.loading_players_for = Some(i);
                                        let (tx, rx) = mpsc::channel();
                                        self.rx_players = Some(rx);
                                        let client1 = client.clone();
                                        let ctx = ui.ctx().clone();
                                        let ep_id = ep.id;
                                        rt.spawn(async move {
                                            let res = client1.get_episode_players(ep_id).await;
                                            let _ = tx.send((i, res));
                                            ctx.request_repaint();
                                        });
                                    }
                                }
                            }

                            if self.selected_episode == Some(i) {
                                ui.indent("players_indent", |ui| {
                                    if self.loading_players_for == Some(i) {
                                        ui.horizontal(|ui| {
                                            ui.spinner();
                                            ui.label("Загрузка плееров...");
                                        });
                                    } else if ep.players.is_empty() {
                                        ui.label("Нет доступных плееров");
                                    }
                                    
                                    for (j, player) in ep.players.iter().enumerate() {
                                        let team_name = player.team.as_ref().map(|t| t.name.as_str()).unwrap_or("Неизвестно");
                                        let player_type = player.player.as_deref().unwrap_or("HLS");
                                        let p_text = format!("{} ({})", team_name, player_type);
                                        
                                        ui.horizontal(|ui| {
                                            ui.radio_value(&mut self.selected_player, Some(j), p_text);
                                            if self.selected_player == Some(j) {
                                                if ui.button("▶ Смотреть").clicked() {
                                                    if let Some(src) = &player.src {
                                                        let src_clone = src.clone();
                                                        let fallback_url = format!("https://v5.animelib.org/anime/{}", self.slug);
                                                        let client_clone = client.clone();
                                                        let process_lock = self.player_process.clone();
                                                        rt.spawn(async move {
                                                            let final_url = if src_clone.contains("kodikplayer.com") {
                                                                match client_clone.extract_kodik_link(&src_clone).await {
                                                                    Ok(u) => u,
                                                                    Err(e) => {
                                                                        log::error!("Kodik extraction failed: {}", e);
                                                                        src_clone
                                                                    }
                                                                }
                                                            } else {
                                                                src_clone
                                                            };
                                                            let child = crate::ui::player_launcher::launch_mpv(&final_url, &fallback_url, hwnd);
                                                            if let Some(c) = child {
                                                                if let Ok(mut lock) = process_lock.lock() {
                                                                    *lock = Some(c);
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                        });
                                    }
                                });
                            }
                        }
                    });
                });
            });
        }
    }
}
