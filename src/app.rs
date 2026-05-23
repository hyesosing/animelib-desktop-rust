// src/app.rs
use crate::api::ApiClient;
use crate::cache::ImageCache;
use crate::ui::{anime_page::AnimePageScreen, catalog::CatalogScreen};
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(Clone, PartialEq)]
pub enum Screen {
    Catalog,
    AnimePage(String),
}

pub enum NavAction {
    None,
    GoTo(Screen),
    GoBack,
}

pub struct AnimeLibApp {
    pub hwnd: Option<isize>,
    rt: Arc<Runtime>,
    api_client: Arc<ApiClient>,
    image_cache: Arc<Mutex<ImageCache>>,
    screen_stack: Vec<Screen>,
    
    catalog_screen: CatalogScreen,
    anime_page_screen: Option<AnimePageScreen>,
    
    search_query: String,
}

impl AnimeLibApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let rt = Arc::new(Runtime::new().expect("Failed to create tokio runtime"));
        let api_client = Arc::new(ApiClient::new());
        let image_cache = Arc::new(Mutex::new(ImageCache::new()));

        Self {
            hwnd: None,
            rt,
            api_client,
            image_cache,
            screen_stack: vec![Screen::Catalog],
            catalog_screen: CatalogScreen::new(),
            anime_page_screen: None,
            search_query: String::new(),
        }
    }

    fn current_screen(&self) -> &Screen {
        self.screen_stack.last().unwrap_or(&Screen::Catalog)
    }

    fn navigate(&mut self, action: NavAction) {
        match action {
            NavAction::None => {}
            NavAction::GoTo(screen) => {
                if let Screen::AnimePage(ref slug) = screen {
                    self.anime_page_screen = Some(AnimePageScreen::new(slug.clone()));
                }
                self.screen_stack.push(screen);
            }
            NavAction::GoBack => {
                if self.screen_stack.len() > 1 {
                    self.screen_stack.pop();
                }
            }
        }
    }
}

impl eframe::App for AnimeLibApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.hwnd.is_none() {
            #[cfg(target_os = "windows")]
            {
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                if let Ok(handle) = _frame.window_handle() {
                    if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                        self.hwnd = Some(win32.hwnd.get() as isize);
                    }
                }
            }
        }

        let mut nav_action = NavAction::None;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if self.screen_stack.len() > 1 {
                    if ui.button("⬅ Назад").clicked() {
                        nav_action = NavAction::GoBack;
                    }
                }

                ui.heading("AnimeLib Desktop");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let response = ui.text_edit_singleline(&mut self.search_query);
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.catalog_screen.set_search(self.search_query.clone());
                        nav_action = NavAction::GoTo(Screen::Catalog);
                    }
                    ui.label("Поиск:");
                });
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_screen() {
                Screen::Catalog => {
                    self.catalog_screen.render(
                        ui,
                        self.api_client.clone(),
                        self.image_cache.clone(),
                        &mut nav_action,
                        self.rt.clone(),
                    );
                }
                Screen::AnimePage(_) => {
                    if let Some(screen) = &mut self.anime_page_screen {
                        screen.render(
                            ui,
                            self.api_client.clone(),
                            self.image_cache.clone(),
                            &mut nav_action,
                            self.rt.clone(),
                            self.hwnd,
                        );
                    }
                }
            }
        });

        self.navigate(nav_action);
    }
}
